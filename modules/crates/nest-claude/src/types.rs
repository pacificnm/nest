//! Shared wire types for the Claude Messages API.
//!
//! v1 covers text, image, tool-use/tool-result, and thinking content blocks.
//! Server-side tool blocks (web search, code execution, etc.) are not yet
//! modeled and will fail to deserialize if a request enables those tools.

use serde::{Deserialize, Serialize};

/// A conversation turn's role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// A user (human or tool-result) turn.
    User,
    /// An assistant (Claude) turn.
    Assistant,
}

/// Ephemeral prompt-cache breakpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheControl {
    #[serde(rename = "type")]
    kind: CacheControlType,
    /// Optional cache time-to-live (defaults to 5 minutes server-side).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<CacheTtl>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum CacheControlType {
    Ephemeral,
}

/// Cache breakpoint time-to-live.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CacheTtl {
    /// 5 minute TTL (default).
    #[serde(rename = "5m")]
    FiveMinutes,
    /// 1 hour TTL.
    #[serde(rename = "1h")]
    OneHour,
}

impl CacheControl {
    /// Creates an ephemeral cache breakpoint with the default (5 minute) TTL.
    pub fn ephemeral() -> Self {
        Self {
            kind: CacheControlType::Ephemeral,
            ttl: None,
        }
    }

    /// Creates an ephemeral cache breakpoint with a 1 hour TTL.
    pub fn ephemeral_1h() -> Self {
        Self {
            kind: CacheControlType::Ephemeral,
            ttl: Some(CacheTtl::OneHour),
        }
    }
}

/// Source for an image content block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ImageSource {
    /// Base64-encoded image bytes.
    Base64 {
        /// MIME type, e.g. `image/png`.
        media_type: String,
        /// Base64-encoded image data.
        data: String,
    },
    /// A remote image URL.
    Url {
        /// The image URL.
        url: String,
    },
}

/// Content for a `tool_result` block: either plain text or nested blocks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolResultContent {
    /// Plain text tool output.
    Text(String),
    /// Structured tool output (text/image blocks).
    Blocks(Vec<ContentBlock>),
}

impl From<String> for ToolResultContent {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for ToolResultContent {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

/// A single content block within a message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    /// Plain text.
    Text {
        /// The text content.
        text: String,
        /// Optional prompt-cache breakpoint.
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    /// An image.
    Image {
        /// The image source.
        source: ImageSource,
        /// Optional prompt-cache breakpoint.
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<CacheControl>,
    },
    /// A model-issued tool call (echoed back on the next turn).
    ToolUse {
        /// The tool call id.
        id: String,
        /// The tool name.
        name: String,
        /// The tool call input.
        input: serde_json::Value,
    },
    /// A client-supplied tool result.
    ToolResult {
        /// The id of the `tool_use` block this result answers.
        tool_use_id: String,
        /// The tool's output.
        content: ToolResultContent,
        /// Whether the tool call failed.
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        is_error: bool,
    },
    /// A model reasoning block (echo back unchanged on the same model).
    Thinking {
        /// The reasoning text (empty when `display` is `"omitted"`).
        thinking: String,
        /// Opaque signature that must be preserved verbatim on replay.
        signature: String,
    },
    /// An encrypted reasoning block with no visible text.
    RedactedThinking {
        /// Opaque encrypted payload.
        data: String,
    },
}

impl ContentBlock {
    /// Creates a plain text block.
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text {
            text: text.into(),
            cache_control: None,
        }
    }

    /// Creates a plain text block with a prompt-cache breakpoint.
    pub fn text_cached(text: impl Into<String>, cache_control: CacheControl) -> Self {
        Self::Text {
            text: text.into(),
            cache_control: Some(cache_control),
        }
    }

    /// Creates a base64-encoded image block.
    pub fn image_base64(media_type: impl Into<String>, data: impl Into<String>) -> Self {
        Self::Image {
            source: ImageSource::Base64 {
                media_type: media_type.into(),
                data: data.into(),
            },
            cache_control: None,
        }
    }

    /// Creates a tool result block.
    pub fn tool_result(
        tool_use_id: impl Into<String>,
        content: impl Into<ToolResultContent>,
    ) -> Self {
        Self::ToolResult {
            tool_use_id: tool_use_id.into(),
            content: content.into(),
            is_error: false,
        }
    }

    /// Creates a failed tool result block.
    pub fn tool_error(
        tool_use_id: impl Into<String>,
        content: impl Into<ToolResultContent>,
    ) -> Self {
        Self::ToolResult {
            tool_use_id: tool_use_id.into(),
            content: content.into(),
            is_error: true,
        }
    }

    /// Returns the text content when this is a [`ContentBlock::Text`] block.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text { text, .. } => Some(text),
            _ => None,
        }
    }
}

/// Why the model stopped generating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// The model finished its response naturally.
    EndTurn,
    /// The response hit the `max_tokens` limit.
    MaxTokens,
    /// The response hit a custom stop sequence.
    StopSequence,
    /// The model wants to call a tool.
    ToolUse,
    /// The model paused a long-running server-side tool loop.
    PauseTurn,
    /// The model refused for safety reasons.
    Refusal,
}

/// Token usage for a request/response.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Usage {
    /// Uncached input tokens processed at full price.
    #[serde(default)]
    pub input_tokens: u32,
    /// Output tokens generated.
    #[serde(default)]
    pub output_tokens: u32,
    /// Tokens written to the prompt cache this request.
    #[serde(default)]
    pub cache_creation_input_tokens: u32,
    /// Tokens served from the prompt cache this request.
    #[serde(default)]
    pub cache_read_input_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_block_round_trips() {
        let block = ContentBlock::text("hello");
        let json = serde_json::to_value(&block).unwrap();
        assert_eq!(json["type"], "text");
        assert_eq!(json["text"], "hello");
        assert!(json.get("cache_control").is_none());

        let parsed: ContentBlock = serde_json::from_value(json).unwrap();
        assert_eq!(parsed.as_text(), Some("hello"));
    }

    #[test]
    fn tool_result_defaults_is_error_false_and_omits_it() {
        let block = ContentBlock::tool_result("toolu_1", "72F and sunny");
        let json = serde_json::to_value(&block).unwrap();
        assert_eq!(json["type"], "tool_result");
        assert_eq!(json["tool_use_id"], "toolu_1");
        assert!(json.get("is_error").is_none());
    }

    #[test]
    fn cache_control_serializes_ttl() {
        let cache = CacheControl::ephemeral_1h();
        let json = serde_json::to_value(&cache).unwrap();
        assert_eq!(json["type"], "ephemeral");
        assert_eq!(json["ttl"], "1h");
    }
}
