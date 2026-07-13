//! Ollama [`nest_ai::AiProvider`] implementation.

#![allow(clippy::result_large_err)]

use async_trait::async_trait;
use futures_util::future::ready;
use futures_util::StreamExt;
use nest_ai::{
    merge_tool_calls, AiProvider, AiResult, CompletionChunk, CompletionRequest, CompletionResponse,
    CompletionStream,
};
use nest_error::NestResult;
use nest_http_client::{HttpClientConfig, HttpClientService};
use tracing::warn;

use crate::client::OllamaClient;
use crate::config::OllamaConfig;
use crate::error::ollama_to_ai_error;
use crate::message::tool_calls_from_ollama;
use crate::shared::OllamaSharedConfig;
use crate::stream::ChatStream;

/// Ollama-backed AI provider.
#[derive(Clone)]
pub struct OllamaProvider {
    client: OllamaClient,
}

impl OllamaProvider {
    /// Creates an Ollama provider with a dedicated HTTP client (tests and standalone use).
    pub fn new(config: OllamaConfig) -> NestResult<Self> {
        let http = HttpClientService::new(
            HttpClientConfig::default().with_user_agent("nest-ai-ollama/0.1"),
        )?;
        Self::with_shared_config(http, OllamaSharedConfig::new(config))
    }

    /// Creates an Ollama provider using the shared [`HttpClientService`].
    pub fn with_http(http: HttpClientService, config: OllamaConfig) -> NestResult<Self> {
        Self::with_shared_config(http, OllamaSharedConfig::new(config))
    }

    /// Creates an Ollama provider bound to a runtime-mutable config handle.
    pub fn with_shared_config(
        http: HttpClientService,
        config: OllamaSharedConfig,
    ) -> NestResult<Self> {
        let client = OllamaClient::new(http, config)?;
        Ok(Self { client })
    }

    /// Returns the underlying client configuration snapshot.
    pub fn config(&self) -> OllamaConfig {
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

        if request.tools.is_empty() {
            let mut stream = self
                .client
                .chat_stream(&request)
                .await
                .map_err(ollama_to_ai_error)?;
            return collect_stream_response(&model, &mut stream).await;
        }

        let response = self
            .client
            .chat(&request)
            .await
            .map_err(ollama_to_ai_error)?;
        let tool_calls = response
            .message
            .tool_calls
            .as_ref()
            .map(|calls| tool_calls_from_ollama(calls))
            .unwrap_or_default();
        let metrics = response.metrics();

        Ok(CompletionResponse {
            model: response.model,
            content: response.message.content,
            done: response.done,
            tool_calls,
            metrics,
        })
    }

    async fn stream_complete(&self, request: CompletionRequest) -> AiResult<CompletionStream> {
        if request.messages.is_empty() {
            return Err(nest_ai::AiError::invalid_input(
                "completion request requires at least one message",
            ));
        }

        let stream = self
            .client
            .chat_stream(&request)
            .await
            .map_err(ollama_to_ai_error)?;

        Ok(map_chat_stream(stream))
    }
}

async fn collect_stream_response(
    model: &str,
    stream: &mut ChatStream,
) -> AiResult<CompletionResponse> {
    let mut content = String::new();
    let mut tool_calls = Vec::new();
    let mut metrics = None;
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(chunk) => {
                content.push_str(&chunk.content);
                if !chunk.tool_calls.is_empty() {
                    merge_tool_calls(&mut tool_calls, &chunk.tool_calls);
                }
                if chunk.metrics.is_some() {
                    metrics = chunk.metrics;
                }
                if chunk.done {
                    break;
                }
            }
            Err(error) => {
                warn!(error = %error, "ollama completion failed");
                return Err(ollama_to_ai_error(error));
            }
        }
    }

    Ok(CompletionResponse {
        model: model.to_string(),
        content,
        done: true,
        tool_calls,
        metrics,
    })
}

fn map_chat_stream(stream: ChatStream) -> CompletionStream {
    Box::pin(
        stream
            .filter(|chunk| {
                ready(matches!(
                    chunk,
                    Ok(item) if !item.content.is_empty()
                        || item.done
                        || !item.tool_calls.is_empty()
                ))
            })
            .map(|chunk| match chunk {
                Ok(chunk) => Ok(CompletionChunk {
                    content_delta: chunk.content,
                    done: chunk.done,
                    metrics: chunk.metrics,
                    tool_calls: chunk.tool_calls,
                }),
                Err(error) => Err(ollama_to_ai_error(error)),
            }),
    )
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
    use nest_ai::ToolDefinition;
    use serde_json::json;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn complete_uses_chat_endpoint() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                "{\"model\":\"smollm2:360m\",\"message\":{\"role\":\"assistant\",\"content\":\"Alien\"},\"done\":true}\n",
            ))
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
        assert!(response.tool_calls.is_empty());
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
                tools: vec![],
            })
            .await
            .unwrap_err();

        assert_eq!(error.kind(), nest_ai::AiErrorKind::InvalidInput);
    }

    #[tokio::test]
    async fn stream_complete_emits_incremental_chunks() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(ResponseTemplate::new(200).set_body_string(concat!(
                "{\"message\":{\"content\":\"Hel\"},\"done\":false}\n",
                "{\"message\":{\"content\":\"lo\"},\"done\":false}\n",
                "{\"message\":{\"content\":\"\"},\"done\":true}\n",
            )))
            .mount(&server)
            .await;

        let config = OllamaConfig::new(server.uri(), "smollm2:360m");
        let provider = OllamaProvider::new(config).unwrap();
        let mut stream = provider
            .stream_complete(CompletionRequest::user_message("hi"))
            .await
            .unwrap();

        let mut content = String::new();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.unwrap();
            content.push_str(&chunk.content_delta);
            if chunk.done {
                break;
            }
        }

        assert_eq!(content, "Hello");
    }

    #[tokio::test]
    async fn complete_with_tools_returns_tool_calls() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "model": "qwen2.5:7b",
                "message": {
                    "role": "assistant",
                    "content": "",
                    "tool_calls": [{
                        "function": {
                            "name": "search_project_memory",
                            "arguments": {"query": "nest-core", "limit": 3}
                        }
                    }]
                },
                "done": true
            })))
            .mount(&server)
            .await;

        let config = OllamaConfig::new(server.uri(), "qwen2.5:7b");
        let provider = OllamaProvider::new(config).unwrap();
        let response = provider
            .complete(
                CompletionRequest::user_message("What is nest-core?").with_tools(vec![
                    ToolDefinition::new(
                        "search_project_memory",
                        "Search project docs",
                        json!({"type": "object"}),
                    ),
                ]),
            )
            .await
            .unwrap();

        assert_eq!(response.tool_calls.len(), 1);
        assert_eq!(response.tool_calls[0].name, "search_project_memory");
        assert_eq!(
            response.tool_calls[0].arguments,
            json!({"query": "nest-core", "limit": 3})
        );
    }

    #[tokio::test]
    async fn stream_complete_emits_tool_calls() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                concat!(
                    "{\"message\":{\"content\":\"\",\"tool_calls\":[{\"function\":{\"name\":\"search_project_memory\",\"arguments\":{\"query\":\"nest\"}}}]},\"done\":false}\n",
                    "{\"message\":{\"content\":\"\"},\"done\":true}\n",
                ),
            ))
            .mount(&server)
            .await;

        let config = OllamaConfig::new(server.uri(), "qwen2.5:7b");
        let provider = OllamaProvider::new(config).unwrap();
        let mut stream = provider
            .stream_complete(CompletionRequest::user_message("search nest"))
            .await
            .unwrap();

        let mut tool_calls = Vec::new();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.unwrap();
            if !chunk.tool_calls.is_empty() {
                merge_tool_calls(&mut tool_calls, &chunk.tool_calls);
            }
            if chunk.done {
                break;
            }
        }

        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].name, "search_project_memory");
    }

    #[tokio::test]
    async fn chat_request_serializes_json_format() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                "{\"model\":\"smollm2:360m\",\"message\":{\"role\":\"assistant\",\"content\":\"{}\"},\"done\":true}\n",
            ))
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
