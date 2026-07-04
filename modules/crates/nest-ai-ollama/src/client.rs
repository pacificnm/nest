//! Low-level Ollama HTTP client.

use nest_ai::{CompletionRequest, ResponseFormat};
use nest_error::NestResult;
use nest_http_client::HttpClientService;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::config::OllamaConfig;
use crate::error::{OllamaError, OllamaResult};
use crate::message::{to_ollama_messages, to_ollama_tools, OllamaChatMessage, OllamaTool};
use crate::shared::OllamaSharedConfig;
use crate::stream::ChatStream;

/// Ollama HTTP client.
#[derive(Clone)]
pub struct OllamaClient {
    http: HttpClientService,
    config: OllamaSharedConfig,
}

impl OllamaClient {
    /// Creates an Ollama client.
    pub fn new(http: HttpClientService, config: OllamaSharedConfig) -> NestResult<Self> {
        Ok(Self { http, config })
    }

    /// Returns the resolved configuration snapshot.
    pub fn config(&self) -> OllamaConfig {
        self.config.snapshot()
    }

    /// Calls `POST /api/chat`.
    pub async fn chat(&self, request: &CompletionRequest) -> OllamaResult<ChatResponse> {
        let model = request
            .model
            .as_deref()
            .unwrap_or(&self.config.snapshot().model)
            .to_string();
        let body = build_request_body(&model, request, false)?;

        let url = format!("{}/api/chat", self.config.snapshot().base_url);
        debug!(model = %model, tools = request.tools.len(), "ollama chat request");
        self.http
            .post_json(&url, &body)
            .await
            .map_err(OllamaError::from)
    }

    /// Calls `POST /api/chat` with streaming enabled.
    pub async fn chat_stream(&self, request: &CompletionRequest) -> OllamaResult<ChatStream> {
        let model = request
            .model
            .as_deref()
            .unwrap_or(&self.config.snapshot().model)
            .to_string();
        let body = build_request_body(&model, request, true)?;

        let url = format!("{}/api/chat", self.config.snapshot().base_url);
        debug!(model = %model, tools = request.tools.len(), "ollama chat stream request");
        let bytes = self
            .http
            .post_json_stream(&url, &body)
            .await
            .map_err(OllamaError::from)?;
        Ok(ChatStream::new(bytes))
    }
}

fn build_request_body(
    model: &str,
    request: &CompletionRequest,
    stream: bool,
) -> OllamaResult<ChatRequestBody> {
    let tools = if request.tools.is_empty() {
        None
    } else {
        Some(to_ollama_tools(&request.tools))
    };
    Ok(ChatRequestBody {
        model: model.to_string(),
        messages: to_ollama_messages(&request.messages),
        stream,
        format: request.format.map(response_format_token),
        tools,
    })
}

#[derive(Debug, Serialize)]
struct ChatRequestBody {
    model: String,
    messages: Vec<OllamaChatMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OllamaTool>>,
}

/// Ollama `POST /api/chat` response body.
#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    /// Model that produced the response.
    pub model: String,
    /// Assistant message payload.
    pub message: OllamaChatMessage,
    /// Whether generation finished.
    pub done: bool,
}

fn response_format_token(format: ResponseFormat) -> &'static str {
    match format {
        ResponseFormat::Text => "text",
        ResponseFormat::Json => "json",
    }
}
