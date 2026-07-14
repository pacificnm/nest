//! `nest_ai::ChatMessage` <-> `nest_claude::Message` conversions.

use std::collections::HashMap;

use nest_ai::{ChatMessage, ChatRole, ToolDefinition};
use nest_claude::{ContentBlock, Message, SystemPrompt};

/// Converts nest-ai chat history into a Claude system prompt (if any
/// [`ChatRole::System`] messages are present) plus the remaining turns as
/// Claude [`Message`]s.
///
/// # Tool-result correlation
///
/// `nest_ai::ChatMessage::tool_result` carries a tool **name**, but Claude's
/// `ContentBlock::tool_result` requires the `tool_use_id` of the specific
/// call it answers. This function recovers that id by walking `messages` in
/// order and remembering, per tool name, the id from the most recent
/// assistant `tool_calls` entry with that name — `nest_ai::ToolCall` carries
/// an id on the assistant side even though the later `Tool`-role message does
/// not.
///
/// **Known limitation:** if an assistant turn calls the same tool name more
/// than once (parallel calls with duplicate names), only the *last* call's id
/// is kept, so a subsequent tool-result message naming that tool always
/// correlates to the last call. This is acceptable for v1 but will silently
/// mis-correlate results for callers that do issue duplicate-name parallel
/// calls in a single turn.
///
/// A `Tool`-role message whose name was never seen in a prior assistant turn
/// (or that has no `tool_name` at all) resolves to an empty `tool_use_id`,
/// which the live Claude API rejects as invalid — callers are expected to
/// supply well-formed history where every tool result follows a matching
/// assistant tool call.
pub fn to_claude_messages(messages: &[ChatMessage]) -> (Option<SystemPrompt>, Vec<Message>) {
    let mut system_parts = Vec::new();
    let mut claude_messages = Vec::new();
    let mut last_tool_use_id_by_name: HashMap<String, String> = HashMap::new();

    for msg in messages {
        match msg.role {
            ChatRole::System => system_parts.push(msg.content.clone()),
            ChatRole::User => claude_messages.push(Message::user(msg.content.clone())),
            ChatRole::Assistant => {
                if let Some(calls) = &msg.tool_calls {
                    let mut blocks = Vec::new();
                    if !msg.content.is_empty() {
                        blocks.push(ContentBlock::text(msg.content.clone()));
                    }
                    for call in calls {
                        last_tool_use_id_by_name.insert(call.name.clone(), call.id.clone());
                        blocks.push(ContentBlock::ToolUse {
                            id: call.id.clone(),
                            name: call.name.clone(),
                            input: call.arguments.clone(),
                        });
                    }
                    claude_messages.push(Message::assistant_blocks(blocks));
                } else {
                    claude_messages.push(Message::assistant(msg.content.clone()));
                }
            }
            ChatRole::Tool => {
                let tool_use_id = msg
                    .tool_name
                    .as_deref()
                    .and_then(|name| last_tool_use_id_by_name.get(name))
                    .cloned()
                    .unwrap_or_default();
                claude_messages.push(Message::user_blocks(vec![ContentBlock::tool_result(
                    tool_use_id,
                    msg.content.clone(),
                )]));
            }
        }
    }

    let system = if system_parts.is_empty() {
        None
    } else {
        Some(SystemPrompt::text(system_parts.join("\n\n")))
    };

    (system, claude_messages)
}

/// Converts nest-ai [`ToolDefinition`]s into Claude's.
pub fn to_claude_tools(tools: &[ToolDefinition]) -> Vec<nest_claude::ToolDefinition> {
    tools
        .iter()
        .map(|tool| {
            nest_claude::ToolDefinition::new(
                tool.name.clone(),
                tool.description.clone(),
                tool.parameters.clone(),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use nest_ai::ToolCall;
    use serde_json::json;

    #[test]
    fn system_messages_are_joined_into_one_prompt() {
        let messages = vec![
            ChatMessage::system("Be terse."),
            ChatMessage::system("Use JSON."),
        ];
        let (system, claude_messages) = to_claude_messages(&messages);
        match system {
            Some(SystemPrompt::Text(text)) => assert_eq!(text, "Be terse.\n\nUse JSON."),
            other => panic!("expected a text system prompt, got {other:?}"),
        }
        assert!(claude_messages.is_empty());
    }

    #[test]
    fn no_system_messages_yields_none() {
        let messages = vec![ChatMessage::user("hi")];
        let (system, _) = to_claude_messages(&messages);
        assert!(system.is_none());
    }

    #[test]
    fn assistant_tool_calls_are_echoed_as_tool_use_blocks() {
        let messages = vec![ChatMessage::assistant_tool_calls(vec![ToolCall::new(
            "get_weather",
            json!({"city": "Paris"}),
        )])];
        let (_, claude_messages) = to_claude_messages(&messages);
        assert_eq!(claude_messages.len(), 1);
        assert!(matches!(
            claude_messages[0].content.as_slice(),
            [ContentBlock::ToolUse { name, .. }] if name == "get_weather"
        ));
    }

    #[test]
    fn tool_result_uses_correct_tool_use_id_for_distinct_names() {
        let mut weather_call = ToolCall::new("get_weather", json!({"city": "Paris"}));
        weather_call.id = "toolu_weather".to_string();
        let mut time_call = ToolCall::new("get_time", json!({"tz": "UTC"}));
        time_call.id = "toolu_time".to_string();

        let messages = vec![
            ChatMessage::assistant_tool_calls(vec![weather_call, time_call]),
            ChatMessage::tool_result("get_weather", "72F and sunny"),
            ChatMessage::tool_result("get_time", "noon"),
        ];

        let (_, claude_messages) = to_claude_messages(&messages);
        assert_eq!(claude_messages.len(), 3);

        let weather_result_id = match &claude_messages[1].content.as_slice() {
            [ContentBlock::ToolResult { tool_use_id, .. }] => tool_use_id.clone(),
            other => panic!("expected a single tool_result block, got {other:?}"),
        };
        let time_result_id = match &claude_messages[2].content.as_slice() {
            [ContentBlock::ToolResult { tool_use_id, .. }] => tool_use_id.clone(),
            other => panic!("expected a single tool_result block, got {other:?}"),
        };

        assert_eq!(weather_result_id, "toolu_weather");
        assert_eq!(time_result_id, "toolu_time");
    }

    #[test]
    fn tool_result_with_unknown_name_falls_back_to_empty_id() {
        let messages = vec![ChatMessage::tool_result("unknown_tool", "result")];
        let (_, claude_messages) = to_claude_messages(&messages);
        match &claude_messages[0].content.as_slice() {
            [ContentBlock::ToolResult { tool_use_id, .. }] => assert_eq!(tool_use_id, ""),
            other => panic!("expected a single tool_result block, got {other:?}"),
        }
    }

    #[test]
    fn maps_tool_definitions_to_claude_shape() {
        let tools = to_claude_tools(&[ToolDefinition::new(
            "search",
            "Search docs",
            json!({"type": "object"}),
        )]);
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "search");
        assert_eq!(tools[0].description, "Search docs");
        assert_eq!(tools[0].input_schema, json!({"type": "object"}));
    }
}
