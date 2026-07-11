//! Request/response types for `POST /v1/messages/count_tokens`.

use serde::{Deserialize, Serialize};

use crate::config::ClaudeConfig;
use crate::request::{
    Message, OutputConfig, SystemPrompt, ThinkingConfig, ToolChoice, ToolDefinition,
};
use crate::types::CacheControl;

/// Builder for `POST /v1/messages/count_tokens`.
///
/// Mirrors [`crate::CreateMessageRequest`] minus `max_tokens`/`stream` (there is
/// no response to cap or stream — this endpoint only counts input tokens).
#[derive(Debug, Clone, Default)]
pub struct CountTokensRequest {
    model: Option<String>,
    messages: Vec<Message>,
    system: Option<SystemPrompt>,
    tools: Option<Vec<ToolDefinition>>,
    tool_choice: Option<ToolChoice>,
    thinking: Option<ThinkingConfig>,
    output_config: Option<OutputConfig>,
    cache_control: Option<CacheControl>,
}

impl CountTokensRequest {
    /// Creates a request builder from conversation history.
    pub fn new(messages: Vec<Message>) -> Self {
        Self {
            messages,
            ..Default::default()
        }
    }

    /// Overrides the model (otherwise [`ClaudeConfig::default_model`] is used).
    ///
    /// Token counts are model-specific — pass the same model you'll use for
    /// the actual `create_message`/`stream_message` call.
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
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

    /// Auto-places a prompt-cache breakpoint on the last cacheable block.
    pub fn cache_control(mut self, cache_control: CacheControl) -> Self {
        self.cache_control = Some(cache_control);
        self
    }

    /// Resolves the default model from `config` and builds the wire body.
    pub(crate) fn into_body(self, config: &ClaudeConfig) -> CountTokensRequestBody {
        CountTokensRequestBody {
            model: self.model.unwrap_or_else(|| config.default_model.clone()),
            messages: self.messages,
            system: self.system,
            tools: self.tools,
            tool_choice: self.tool_choice,
            thinking: self.thinking,
            output_config: self.output_config,
            cache_control: self.cache_control,
        }
    }
}

/// Wire body for `POST /v1/messages/count_tokens`.
#[derive(Debug, Clone, Serialize)]
pub(crate) struct CountTokensRequestBody {
    model: String,
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
    cache_control: Option<CacheControl>,
}

/// Response from `POST /v1/messages/count_tokens`.
#[derive(Debug, Clone, Deserialize)]
pub struct TokenCountResponse {
    /// The total number of tokens across the messages, system prompt, and tools.
    pub input_tokens: u32,
    /// Present when context management was applied to the (hypothetical) request.
    pub context_management: Option<ContextManagementTokenInfo>,
}

/// Token counts before context management edits, when applicable.
#[derive(Debug, Clone, Deserialize)]
pub struct ContextManagementTokenInfo {
    /// The token count before context management was applied.
    pub original_input_tokens: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> ClaudeConfig {
        ClaudeConfig::builder().api_key("test-key").build().unwrap()
    }

    #[test]
    fn resolves_default_model_and_omits_optional_fields() {
        let request = CountTokensRequest::new(vec![Message::user("hi")]);
        let body = request.into_body(&test_config());
        assert_eq!(body.model, "claude-opus-4-8");

        let json = serde_json::to_value(&body).unwrap();
        assert!(json.get("system").is_none());
        assert!(json.get("tools").is_none());
        assert!(json.get("max_tokens").is_none());
        assert!(json.get("stream").is_none());
    }

    #[test]
    fn explicit_model_overrides_default() {
        let request = CountTokensRequest::new(vec![Message::user("hi")]).model("claude-sonnet-5");
        let body = request.into_body(&test_config());
        assert_eq!(body.model, "claude-sonnet-5");
    }

    #[test]
    fn response_parses_without_context_management() {
        let response: TokenCountResponse =
            serde_json::from_value(serde_json::json!({"input_tokens": 42})).unwrap();
        assert_eq!(response.input_tokens, 42);
        assert!(response.context_management.is_none());
    }

    #[test]
    fn response_parses_with_context_management() {
        let response: TokenCountResponse = serde_json::from_value(serde_json::json!({
            "input_tokens": 2095,
            "context_management": {"original_input_tokens": 4000}
        }))
        .unwrap();
        assert_eq!(response.input_tokens, 2095);
        assert_eq!(
            response.context_management.unwrap().original_input_tokens,
            4000
        );
    }
}
