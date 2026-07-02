//! Ollama [`nest_ai::AiProvider`] implementation.

use async_trait::async_trait;
use nest_ai::{AiProvider, AiResult, CompletionRequest, CompletionResponse};
use nest_error::NestResult;
use nest_http_client::{HttpClientConfig, HttpClientService};
use tracing::warn;

use crate::client::OllamaClient;
use crate::config::OllamaConfig;
use crate::error::ollama_to_ai_error;

/// Ollama-backed AI provider.
#[derive(Clone)]
pub struct OllamaProvider {
    client: OllamaClient,
}

impl OllamaProvider {
    /// Creates an Ollama provider from configuration.
    pub fn new(config: OllamaConfig) -> NestResult<Self> {
        let http = HttpClientService::new(
            HttpClientConfig::default().with_user_agent("nest-ai-ollama/0.1"),
        )?;
        let client = OllamaClient::new(http, config)?;
        Ok(Self { client })
    }

    /// Returns the underlying client configuration.
    pub fn config(&self) -> &OllamaConfig {
        self.client.config()
    }
}

#[async_trait]
impl AiProvider for OllamaProvider {
    fn provider_id(&self) -> &'static str {
        "ollama"
    }

    async fn complete(&self, request: CompletionRequest) -> AiResult<CompletionResponse> {
        if request.messages.is_empty() {
            return Err(nest_ai::AiError::invalid_input(
                "completion request requires at least one message",
            ));
        }

        let model = request
            .model
            .as_deref()
            .unwrap_or(&self.client.config().model)
            .to_string();

        match self
            .client
            .chat(&model, &request.messages, request.format)
            .await
        {
            Ok(response) => Ok(CompletionResponse {
                model: response.model,
                content: response.message.content,
                done: response.done,
            }),
            Err(error) => {
                warn!(error = %error, "ollama completion failed");
                Err(ollama_to_ai_error(error))
            }
        }
    }
}

/// Probes whether Ollama responds at the configured base URL.
pub async fn is_available(config: &OllamaConfig) -> bool {
    let provider = match OllamaProvider::new(config.clone()) {
        Ok(provider) => provider,
        Err(_) => return false,
    };

    provider
        .complete(CompletionRequest::user_message("ping").with_model(&config.model))
        .await
        .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn complete_uses_chat_endpoint() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "model": "smollm2:360m",
                "message": { "role": "assistant", "content": "Alien" },
                "done": true
            })))
            .mount(&server)
            .await;

        let config = OllamaConfig::new(server.uri(), "smollm2:360m");
        let provider = OllamaProvider::new(config).unwrap();
        let response = provider
            .complete(CompletionRequest::user_message("Filename: Alien.mkv"))
            .await
            .unwrap();

        assert_eq!(response.content, "Alien");
        assert_eq!(response.model, "smollm2:360m");
        assert!(response.done);
    }

    #[tokio::test]
    async fn complete_rejects_empty_messages() {
        let server = MockServer::start().await;
        let config = OllamaConfig::new(server.uri(), "smollm2:360m");
        let provider = OllamaProvider::new(config).unwrap();
        let error = provider
            .complete(CompletionRequest {
                model: None,
                messages: vec![],
                format: None,
            })
            .await
            .unwrap_err();

        assert_eq!(error.kind(), nest_ai::AiErrorKind::InvalidInput);
    }

    #[tokio::test]
    async fn chat_request_serializes_json_format() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "model": "smollm2:360m",
                "message": { "role": "assistant", "content": "{}" },
                "done": true
            })))
            .mount(&server)
            .await;

        let config = OllamaConfig::new(server.uri(), "smollm2:360m");
        let provider = OllamaProvider::new(config).unwrap();
        provider
            .complete(
                CompletionRequest::user_message("test")
                    .with_model("smollm2:360m")
                    .with_json_format(),
            )
            .await
            .unwrap();
    }
}
