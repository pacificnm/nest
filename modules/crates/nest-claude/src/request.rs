//! Request-side types for `POST /v1/messages`.

use serde::{Deserialize, Serialize};

use crate::config::ClaudeConfig;
use crate::types::{CacheControl, ContentBlock, Role};

/// A single conversation turn.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// The turn's role.
    pub role: Role,
    /// The turn's content blocks.
    pub content: Vec<ContentBlock>,
}

impl Message {
    /// Creates a user turn from plain text.
    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: vec![ContentBlock::text(text)],
        }
    }

    /// Creates a user turn from explicit content blocks.
    pub fn user_blocks(content: Vec<ContentBlock>) -> Self {
        Self {
            role: Role::User,
            content,
        }
    }

    /// Creates an assistant turn from plain text.
    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: vec![ContentBlock::text(text)],
        }
    }

    /// Creates an assistant turn from explicit content blocks (e.g. echoing a
    /// prior response, including `tool_use`/`thinking` blocks, back into history).
    pub fn assistant_blocks(content: Vec<ContentBlock>) -> Self {
        Self {
            role: Role::Assistant,
            content,
        }
    }
}

/// System prompt: plain text or cache-annotated text blocks.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SystemPrompt {
    /// A plain-text system prompt.
    Text(String),
    /// One or more text blocks, e.g. to attach a prompt-cache breakpoint.
    Blocks(Vec<ContentBlock>),
}

impl SystemPrompt {
    /// Creates a plain-text system prompt.
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text(text.into())
    }

    /// Creates a system prompt with a prompt-cache breakpoint on the text.
    pub fn cached(text: impl Into<String>, cache_control: CacheControl) -> Self {
        Self::Blocks(vec![ContentBlock::text_cached(text, cache_control)])
    }
}

impl From<&str> for SystemPrompt {
    fn from(value: &str) -> Self {
        Self::text(value)
    }
}

impl From<String> for SystemPrompt {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

/// A tool definition offered to the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    /// The tool's name.
    pub name: String,
    /// A description of when and how to use the tool.
    pub description: String,
    /// JSON Schema for the tool's input.
    pub input_schema: serde_json::Value,
    /// Optional prompt-cache breakpoint (place on the last tool definition).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<CacheControl>,
}

impl ToolDefinition {
    /// Creates a tool definition.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: serde_json::Value,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema,
            cache_control: None,
        }
    }

    /// Attaches a prompt-cache breakpoint to this tool definition.
    pub fn with_cache_control(mut self, cache_control: CacheControl) -> Self {
        self.cache_control = Some(cache_control);
        self
    }
}

/// Controls whether/which tool the model must call.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ToolChoice {
    /// The model decides whether to use tools (default).
    Auto,
    /// The model must use at least one tool.
    Any,
    /// The model must use the named tool.
    Tool {
        /// The tool name to force.
        name: String,
    },
    /// The model cannot use tools.
    None,
}

/// How visible reasoning is in the response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThinkingDisplay {
    /// Thinking blocks stream with empty text (default on current models).
    Omitted,
    /// Thinking blocks carry a readable summary of the reasoning.
    Summarized,
}

/// Extended-thinking configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ThinkingConfig {
    /// Claude decides when and how much to think (current models).
    Adaptive {
        /// Controls whether reasoning is surfaced in the response.
        #[serde(skip_serializing_if = "Option::is_none")]
        display: Option<ThinkingDisplay>,
    },
    /// Thinking is turned off.
    Disabled,
    /// Fixed thinking token budget (older, pre-4.6 models only).
    Enabled {
        /// Thinking token budget; must be less than `max_tokens`.
        budget_tokens: u32,
    },
}

impl ThinkingConfig {
    /// Adaptive thinking with the default (omitted) display.
    pub fn adaptive() -> Self {
        Self::Adaptive { display: None }
    }

    /// Adaptive thinking with a readable summary streamed back.
    pub fn adaptive_summarized() -> Self {
        Self::Adaptive {
            display: Some(ThinkingDisplay::Summarized),
        }
    }
}

/// Thinking-depth / token-spend control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Effort {
    /// Lowest depth; short, scoped, latency-sensitive tasks.
    Low,
    /// Cost-saving step-down from `high`.
    Medium,
    /// Default; balances token usage and intelligence.
    High,
    /// Best for coding and agentic use cases.
    Xhigh,
    /// Highest depth; intelligence-demanding tasks with no token constraint.
    Max,
}

/// Output-shaping configuration (effort, structured output format).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OutputConfig {
    /// Thinking-depth / token-spend control.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<Effort>,
}

/// Builder for a `POST /v1/messages` request.
///
/// `model` and `max_tokens` fall back to [`ClaudeConfig::default_model`] and
/// [`ClaudeConfig::default_max_tokens`] when left unset.
#[derive(Debug, Clone, Default)]
pub struct CreateMessageRequest {
    model: Option<String>,
    max_tokens: Option<u32>,
    messages: Vec<Message>,
    system: Option<SystemPrompt>,
    tools: Option<Vec<ToolDefinition>>,
    tool_choice: Option<ToolChoice>,
    thinking: Option<ThinkingConfig>,
    output_config: Option<OutputConfig>,
    stop_sequences: Option<Vec<String>>,
    cache_control: Option<CacheControl>,
}

impl CreateMessageRequest {
    /// Creates a request builder from conversation history.
    pub fn new(messages: Vec<Message>) -> Self {
        Self {
            messages,
            ..Default::default()
        }
    }

    /// Overrides the model (otherwise [`ClaudeConfig::default_model`] is used).
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Overrides `max_tokens` (otherwise [`ClaudeConfig::default_max_tokens`] is used).
    pub fn max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Sets the system prompt.
    pub fn system(mut self, system: impl Into<SystemPrompt>) -> Self {
        self.system = Some(system.into());
        self
    }

    /// Sets the available tools.
    pub fn tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Forces tool-use behavior.
    pub fn tool_choice(mut self, tool_choice: ToolChoice) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    /// Enables extended thinking.
    pub fn thinking(mut self, thinking: ThinkingConfig) -> Self {
        self.thinking = Some(thinking);
        self
    }

    /// Sets output-shaping configuration (e.g. `effort`).
    pub fn output_config(mut self, output_config: OutputConfig) -> Self {
        self.output_config = Some(output_config);
        self
    }

    /// Sets custom stop sequences.
    pub fn stop_sequences(mut self, stop_sequences: Vec<String>) -> Self {
        self.stop_sequences = Some(stop_sequences);
        self
    }

    /// Auto-places a prompt-cache breakpoint on the last cacheable block.
    pub fn cache_control(mut self, cache_control: CacheControl) -> Self {
        self.cache_control = Some(cache_control);
        self
    }

    /// Resolves defaults from `config` and builds the request body sent on the wire.
    pub(crate) fn into_body(self, config: &ClaudeConfig, stream: bool) -> MessageRequestBody {
        MessageRequestBody {
            model: self.model.unwrap_or_else(|| config.default_model.clone()),
            max_tokens: self.max_tokens.unwrap_or(config.default_max_tokens),
            messages: self.messages,
            system: self.system,
            tools: self.tools,
            tool_choice: self.tool_choice,
            thinking: self.thinking,
            output_config: self.output_config,
            stop_sequences: self.stop_sequences,
            cache_control: self.cache_control,
            stream,
        }
    }
}

/// Wire body for `POST /v1/messages` (required fields resolved from config).
#[derive(Debug, Clone, Serialize)]
pub(crate) struct MessageRequestBody {
    model: String,
    max_tokens: u32,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<SystemPrompt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ToolDefinition>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<ToolChoice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking: Option<ThinkingConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_config: Option<OutputConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cache_control: Option<CacheControl>,
    stream: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> ClaudeConfig {
        ClaudeConfig::builder().api_key("test-key").build().unwrap()
    }

    #[test]
    fn resolves_default_model_and_max_tokens() {
        let request = CreateMessageRequest::new(vec![Message::user("hi")]);
        let body = request.into_body(&test_config(), false);
        assert_eq!(body.model, "claude-opus-4-8");
        assert_eq!(body.max_tokens, crate::config::DEFAULT_MAX_TOKENS);
        assert!(!body.stream);
    }

    #[test]
    fn explicit_model_overrides_default() {
        let request = CreateMessageRequest::new(vec![Message::user("hi")])
            .model("claude-sonnet-5")
            .max_tokens(1024);
        let body = request.into_body(&test_config(), true);
        assert_eq!(body.model, "claude-sonnet-5");
        assert_eq!(body.max_tokens, 1024);
        assert!(body.stream);
    }

    #[test]
    fn serializes_without_null_optional_fields() {
        let request = CreateMessageRequest::new(vec![Message::user("hi")]);
        let body = request.into_body(&test_config(), false);
        let json = serde_json::to_value(&body).unwrap();
        assert!(json.get("system").is_none());
        assert!(json.get("tools").is_none());
        assert!(json.get("thinking").is_none());
    }
}
