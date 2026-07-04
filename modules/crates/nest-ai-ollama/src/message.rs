//! Ollama chat message and tool wire-format mapping.

use nest_ai::{ChatMessage, ChatRole, ToolCall, ToolDefinition};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Ollama `POST /api/chat` message body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaChatMessage {
    /// Message role.
    pub role: ChatRole,
    /// Text content.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub content: String,
    /// Tool name for tool-role messages.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    /// Tool calls requested by the assistant.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OllamaToolCall>>,
}

/// Ollama tool call payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaToolCall {
    /// Nested function invocation.
    pub function: OllamaFunctionCall,
}

/// Ollama function call payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaFunctionCall {
    /// Function name.
    pub name: String,
    /// JSON arguments object or string fragment.
    #[serde(default)]
    pub arguments: Value,
}

/// Ollama tool definition entry.
#[derive(Debug, Clone, Serialize)]
pub struct OllamaTool {
    /// Entry type (`function`).
    #[serde(rename = "type")]
    pub kind: &'static str,
    /// Function schema.
    pub function: OllamaFunctionDefinition,
}

/// Ollama function schema.
#[derive(Debug, Clone, Serialize)]
pub struct OllamaFunctionDefinition {
    /// Function name.
    pub name: String,
    /// Description for the model.
    pub description: String,
    /// JSON Schema parameters object.
    pub parameters: Value,
}

/// Converts Nest chat messages into Ollama wire messages.
pub fn to_ollama_messages(messages: &[ChatMessage]) -> Vec<OllamaChatMessage> {
    messages
        .iter()
        .map(|message| OllamaChatMessage {
            role: message.role,
            content: message.content.clone(),
            tool_name: message.tool_name.clone(),
            tool_calls: message.tool_calls.as_ref().map(|calls| {
                calls
                    .iter()
                    .map(|call| OllamaToolCall {
                        function: OllamaFunctionCall {
                            name: call.name.clone(),
                            arguments: call.arguments.clone(),
                        },
                    })
                    .collect()
            }),
        })
        .collect()
}

/// Converts Nest tool definitions into Ollama tool entries.
pub fn to_ollama_tools(tools: &[ToolDefinition]) -> Vec<OllamaTool> {
    tools
        .iter()
        .map(|tool| OllamaTool {
            kind: "function",
            function: OllamaFunctionDefinition {
                name: tool.name.clone(),
                description: tool.description.clone(),
                parameters: tool.parameters.clone(),
            },
        })
        .collect()
}

/// Parses tool calls from an Ollama assistant message payload.
pub fn tool_calls_from_ollama(calls: &[OllamaToolCall]) -> Vec<ToolCall> {
    calls
        .iter()
        .map(|call| ToolCall {
            id: String::new(),
            name: call.function.name.clone(),
            arguments: normalize_arguments(&call.function.arguments),
        })
        .collect()
}

fn normalize_arguments(arguments: &Value) -> Value {
    match arguments {
        Value::String(raw) if !raw.is_empty() => {
            serde_json::from_str(raw).unwrap_or_else(|_| Value::String(raw.clone()))
        }
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn round_trip_tool_result_message() {
        let message = ChatMessage::tool_result("search_project_memory", "results");
        let wire = to_ollama_messages(&[message]);
        assert_eq!(wire[0].role, ChatRole::Tool);
        assert_eq!(wire[0].tool_name.as_deref(), Some("search_project_memory"));
    }

    #[test]
    fn maps_tool_definitions_to_ollama_shape() {
        let tools = to_ollama_tools(&[ToolDefinition::new(
            "search",
            "Search docs",
            json!({"type": "object"}),
        )]);
        assert_eq!(tools[0].kind, "function");
        assert_eq!(tools[0].function.name, "search");
    }
}
