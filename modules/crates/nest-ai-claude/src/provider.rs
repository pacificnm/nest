//! `ClaudeAiProvider`: `nest_ai::AiProvider` implementation over `nest_claude::ClaudeClient`.

use async_trait::async_trait;
use nest_ai::{
    AiError, AiProvider, AiResult, CompletionRequest, CompletionResponse, ResponseFormat, ToolCall,
};
use nest_claude::{ClaudeClient, ClaudeConfig, CreateMessageRequest};
use nest_error::NestResult;

use crate::error::claude_to_ai_error;
use crate::message::{to_claude_messages, to_claude_tools};

/// Claude-backed AI provider.
///
/// Owns a dedicated [`ClaudeClient`] (and, through it, a dedicated
/// `HttpClientService`) rather than the app-wide shared HTTP client, since
/// the Claude API key must not leak into other modules' requests.
#[derive(Clone)]
pub struct ClaudeAiProvider {
    client: ClaudeClient,
}

impl ClaudeAiProvider {
    /// Creates a Claude provider from resolved configuration.
    pub fn new(config: ClaudeConfig) -> NestResult<Self> {
        let client = ClaudeClient::new(config).map_err(nest_error::NestError::from)?;
        Ok(Self { client })
    }
}

#[async_trait]
impl AiProvider for ClaudeAiProvider {
    fn provider_id(&self) -> &'static str {
        "claude"
    }

    async fn complete(&self, request: CompletionRequest) -> AiResult<CompletionResponse> {
        if request.messages.is_empty() {
            return Err(AiError::invalid_input(
                "completion request requires at least one message",
            ));
        }
        if request.format == Some(ResponseFormat::Json) {
            // Claude's Messages API has no direct "JSON mode" equivalent to
            // Ollama's `format: "json"` — being explicit here beats silently
            // ignoring the hint and returning prose the caller didn't ask for.
            return Err(AiError::invalid_input(
                "JSON response format is not yet supported by nest-ai-claude",
            ));
        }

        let (system, messages) = to_claude_messages(&request.messages);
        let mut claude_request = CreateMessageRequest::new(messages);
        if let Some(system) = system {
            claude_request = claude_request.system(system);
        }
        if let Some(model) = &request.model {
            claude_request = claude_request.model(model);
        }
        if !request.tools.is_empty() {
            claude_request = claude_request.tools(to_claude_tools(&request.tools));
        }

        let response = self
            .client
            .create_message(claude_request)
            .await
            .map_err(claude_to_ai_error)?;

        let tool_calls: Vec<ToolCall> = response
            .tool_uses()
            .map(|(id, name, input)| ToolCall {
                id: id.to_string(),
                name: name.to_string(),
                arguments: input.clone(),
            })
            .collect();

        Ok(CompletionResponse {
            model: response.model.clone(),
            content: response.text(),
            done: true,
            tool_calls,
            // `nest_claude::Usage` only carries token counts (input/output/
            // cache_creation/cache_read) - Claude's API reports no timing or
            // throughput data, so there's nothing to populate
            // `CompletionMetrics`'s prompt_tps/generation_tps/*_time_ms
            // fields with. Left `None` for v1 rather than fabricating zeros.
            metrics: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use nest_ai::ToolDefinition;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;

    fn test_config(base_url: String) -> ClaudeConfig {
        ClaudeConfig::builder()
            .api_key("test-key")
            .base_url(base_url)
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn complete_uses_messages_endpoint() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "msg_1",
                "type": "message",
                "role": "assistant",
                "content": [{"type": "text", "text": "Hello, world."}],
                "model": "claude-opus-4-8",
                "stop_reason": "end_turn",
                "stop_sequence": null,
                "usage": {"input_tokens": 10, "output_tokens": 5}
            })))
            .mount(&server)
            .await;

        let provider = ClaudeAiProvider::new(test_config(server.uri())).unwrap();
        let response = provider
            .complete(CompletionRequest::user_message("hi"))
            .await
            .unwrap();

        assert_eq!(response.content, "Hello, world.");
        assert_eq!(response.model, "claude-opus-4-8");
        assert!(response.done);
        assert!(response.tool_calls.is_empty());
    }

    #[tokio::test]
    async fn complete_rejects_empty_messages() {
        let server = MockServer::start().await;
        let provider = ClaudeAiProvider::new(test_config(server.uri())).unwrap();
        let error = provider
            .complete(CompletionRequest {
                model: None,
                messages: vec![],
                format: None,
                tools: vec![],
            })
            .await
            .unwrap_err();

        assert_eq!(error.kind(), nest_ai::AiErrorKind::InvalidInput);
    }

    #[tokio::test]
    async fn complete_rejects_json_format() {
        let server = MockServer::start().await;
        let provider = ClaudeAiProvider::new(test_config(server.uri())).unwrap();
        let error = provider
            .complete(CompletionRequest::user_message("hi").with_json_format())
            .await
            .unwrap_err();

        assert_eq!(error.kind(), nest_ai::AiErrorKind::InvalidInput);
    }

    #[tokio::test]
    async fn complete_with_tools_returns_tool_calls() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "msg_1",
                "type": "message",
                "role": "assistant",
                "content": [{
                    "type": "tool_use",
                    "id": "toolu_1",
                    "name": "get_weather",
                    "input": {"city": "Paris"}
                }],
                "model": "claude-opus-4-8",
                "stop_reason": "tool_use",
                "stop_sequence": null,
                "usage": {"input_tokens": 10, "output_tokens": 5}
            })))
            .mount(&server)
            .await;

        let provider = ClaudeAiProvider::new(test_config(server.uri())).unwrap();
        let response = provider
            .complete(
                CompletionRequest::user_message("weather in Paris?").with_tools(vec![
                    ToolDefinition::new(
                        "get_weather",
                        "Gets the weather",
                        json!({"type": "object"}),
                    ),
                ]),
            )
            .await
            .unwrap();

        assert_eq!(response.tool_calls.len(), 1);
        assert_eq!(response.tool_calls[0].id, "toolu_1");
        assert_eq!(response.tool_calls[0].name, "get_weather");
        assert_eq!(response.tool_calls[0].arguments, json!({"city": "Paris"}));
    }
}
