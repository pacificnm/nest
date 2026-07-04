//! Tool-call argument validation and content fallbacks.

use nest_ai::ToolCall;
use serde_json::Value;

/// Returns true when `arguments` look like a JSON Schema fragment rather than values.
pub fn looks_like_schema_arguments(arguments: &Value) -> bool {
    let Value::Object(map) = arguments else {
        return false;
    };
    if map.is_empty() {
        return false;
    }
    map.values().all(|value| {
        value.as_object().is_some_and(|object| {
            object.contains_key("type")
                || object.contains_key("anyOf")
                || object.contains_key("properties")
                || object.contains_key("items")
        })
    })
}

/// Validates tool arguments before MCP execution.
pub fn validate_tool_arguments(arguments: &Value) -> Result<(), String> {
    if looks_like_schema_arguments(arguments) {
        return Err(
            "model returned JSON Schema instead of argument values; pass concrete strings and numbers"
                .into(),
        );
    }
    Ok(())
}

/// Parses a tool call the model may have emitted as assistant text instead of structured output.
pub fn parse_tool_calls_from_content(content: &str) -> Option<Vec<ToolCall>> {
    for candidate in json_candidates_from_content(content) {
        if let Some(calls) = parse_tool_call_value(&candidate) {
            return Some(calls);
        }
    }
    None
}

fn json_candidates_from_content(content: &str) -> Vec<Value> {
    let mut seen = Vec::new();
    let mut candidates = Vec::new();

    let mut push_json = |text: &str| {
        let trimmed = text.trim();
        if trimmed.is_empty() || seen.iter().any(|prior| prior == trimmed) {
            return;
        }
        seen.push(trimmed.to_string());
        if let Ok(value) = serde_json::from_str(trimmed) {
            candidates.push(value);
        }
    };

    let trimmed = content.trim();
    push_json(trimmed);

    if let Some(inner) = extract_fenced_json(trimmed) {
        push_json(&inner);
    }

    if let Some(object) = extract_balanced_json_object(trimmed) {
        push_json(&object);
    }

    candidates
}

fn extract_fenced_json(content: &str) -> Option<String> {
    let fence_start = content.find("```")?;
    let after_open = &content[fence_start + 3..];
    let after_lang = after_open
        .strip_prefix("json")
        .or_else(|| after_open.strip_prefix("JSON"))
        .unwrap_or(after_open);
    let body = after_lang.trim_start();
    let fence_end = body.find("```")?;
    Some(body[..fence_end].trim().to_string())
}

fn extract_balanced_json_object(content: &str) -> Option<String> {
    let start = content.find('{')?;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (offset, byte) in content[start..].bytes().enumerate() {
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            continue;
        }

        match byte {
            b'"' => in_string = true,
            b'{' => depth += 1,
            b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let end = start + offset + 1;
                    return Some(content[start..end].to_string());
                }
            }
            _ => {}
        }
    }

    None
}

fn parse_tool_call_value(value: &Value) -> Option<Vec<ToolCall>> {
    match value {
        Value::Array(items) => {
            let calls: Vec<_> = items.iter().filter_map(parse_single_tool_call).collect();
            if calls.is_empty() {
                None
            } else {
                Some(calls)
            }
        }
        other => parse_single_tool_call(other).map(|call| vec![call]),
    }
}

fn parse_single_tool_call(value: &Value) -> Option<ToolCall> {
    let object = value.as_object()?;

    if let Some(name) = object.get("name").and_then(Value::as_str) {
        let arguments = object
            .get("arguments")
            .cloned()
            .unwrap_or(Value::Object(Default::default()));
        return Some(ToolCall::new(name, arguments));
    }

    let function = object.get("function")?.as_object()?;
    let name = function.get("name")?.as_str()?;
    let arguments = function
        .get("arguments")
        .cloned()
        .unwrap_or(Value::Object(Default::default()));
    Some(ToolCall::new(name, arguments))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn detects_schema_like_arguments() {
        assert!(looks_like_schema_arguments(&json!({
            "content": {"type": "string"},
            "session_key": {"type": "string"}
        })));
    }

    #[test]
    fn accepts_concrete_arguments() {
        assert!(!looks_like_schema_arguments(&json!({
            "query": "nest-core",
            "limit": 3
        })));
    }

    #[test]
    fn parses_tool_call_json_from_content() {
        let calls = parse_tool_calls_from_content(
            r#"{"name":"nest_memory__search_project_memory","arguments":{"query":"nest"}}"#,
        )
        .unwrap();
        assert_eq!(calls[0].name, "nest_memory__search_project_memory");
        assert_eq!(calls[0].arguments, json!({"query": "nest"}));
    }

    #[test]
    fn parses_tool_call_from_markdown_fence() {
        let calls = parse_tool_calls_from_content(
            "```json\n{\"name\":\"nest_knowledge__list_knowledge_collections\",\"arguments\":{}}\n```",
        )
        .unwrap();
        assert_eq!(calls[0].name, "nest_knowledge__list_knowledge_collections");
        assert_eq!(calls[0].arguments, json!({}));
    }

    #[test]
    fn parses_ollama_function_shape_from_content() {
        let calls = parse_tool_calls_from_content(
            r#"{"function":{"name":"list_knowledge_collections","arguments":{}}}"#,
        )
        .unwrap();
        assert_eq!(calls[0].name, "list_knowledge_collections");
    }

    #[test]
    fn parses_tool_call_embedded_in_prose() {
        let calls = parse_tool_calls_from_content(
            "I'll list collections:\n```json\n{\"name\":\"nest_knowledge__list_knowledge_collections\",\"arguments\":{}}\n```",
        )
        .unwrap();
        assert_eq!(calls[0].name, "nest_knowledge__list_knowledge_collections");
    }
}
