//! Response-side types for `POST /v1/messages`.

use serde::Deserialize;

use crate::types::{ContentBlock, Role, StopReason, Usage};

/// A `POST /v1/messages` response.
#[derive(Debug, Clone, Deserialize)]
pub struct MessageResponse {
    /// The message id.
    pub id: String,
    /// The turn's role (always `assistant`).
    pub role: Role,
    /// The response content blocks.
    pub content: Vec<ContentBlock>,
    /// The model that produced the response.
    pub model: String,
    /// Why generation stopped, if the response is complete.
    pub stop_reason: Option<StopReason>,
    /// The custom stop sequence hit, if any.
    pub stop_sequence: Option<String>,
    /// Token usage for this request.
    pub usage: Usage,
}

impl MessageResponse {
    /// Concatenates all `text` content blocks, in order.
    pub fn text(&self) -> String {
        self.content
            .iter()
            .filter_map(ContentBlock::as_text)
            .collect::<Vec<_>>()
            .join("")
    }

    /// Returns the `tool_use` content blocks, in order.
    pub fn tool_uses(&self) -> impl Iterator<Item = (&str, &str, &serde_json::Value)> {
        self.content.iter().filter_map(|block| match block {
            ContentBlock::ToolUse { id, name, input } => Some((id.as_str(), name.as_str(), input)),
            _ => None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_concatenates_text_blocks() {
        let response: MessageResponse = serde_json::from_value(serde_json::json!({
            "id": "msg_1",
            "type": "message",
            "role": "assistant",
            "content": [
                {"type": "text", "text": "Hello, "},
                {"type": "text", "text": "world."}
            ],
            "model": "claude-opus-4-8",
            "stop_reason": "end_turn",
            "stop_sequence": null,
            "usage": {"input_tokens": 10, "output_tokens": 5}
        }))
        .unwrap();

        assert_eq!(response.text(), "Hello, world.");
        assert_eq!(response.stop_reason, Some(StopReason::EndTurn));
    }

    #[test]
    fn tool_uses_extracts_blocks() {
        let response: MessageResponse = serde_json::from_value(serde_json::json!({
            "id": "msg_1",
            "type": "message",
            "role": "assistant",
            "content": [
                {"type": "tool_use", "id": "toolu_1", "name": "get_weather", "input": {"city": "Paris"}}
            ],
            "model": "claude-opus-4-8",
            "stop_reason": "tool_use",
            "stop_sequence": null,
            "usage": {"input_tokens": 10, "output_tokens": 5}
        }))
        .unwrap();

        let calls: Vec<_> = response.tool_uses().collect();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, "toolu_1");
        assert_eq!(calls[0].1, "get_weather");
    }
}
