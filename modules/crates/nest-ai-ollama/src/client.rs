//! Low-level Ollama HTTP client.

use nest_ai::{ChatMessage, ResponseFormat};
use nest_error::NestResult;
use nest_http_client::HttpClientService;
use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::config::OllamaConfig;
use crate::error::{OllamaError, OllamaResult};

/// Ollama HTTP client.
#[derive(Clone)]
pub struct OllamaClient {
    http: HttpClientService,
    config: OllamaConfig,
}

impl OllamaClient {
    /// Creates an Ollama client.
    pub fn new(http: HttpClientService, config: OllamaConfig) -> NestResult<Self> {
        Ok(Self { http, config })
    }

    /// Returns the resolved configuration.
    pub fn config(&self) -> &OllamaConfig {
        &self.config
    }

    /// Calls `POST /api/chat`.
    pub async fn chat(
        &self,
        model: &str,
        messages: &[ChatMessage],
        format: Option<ResponseFormat>,
    ) -> OllamaResult<ChatResponse> {
        let body = ChatRequestBody {
            model: model.to_string(),
            messages: messages.to_vec(),
            stream: false,
            format: format.map(response_format_token),
        };

        let url = format!("{}/api/chat", self.config.base_url);
        debug!(model = %model, "ollama chat request");
        self.http
            .post_json(&url, &body)
            .await
            .map_err(OllamaError::from)
    }
}

#[derive(Debug, Serialize)]
struct ChatRequestBody {
    model: String,
    messages: Vec<ChatMessage>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    format: Option<&'static str>,
}

/// Ollama `POST /api/chat` response body.
#[derive(Debug, Deserialize)]
pub struct ChatResponse {
    /// Model that produced the response.
    pub model: String,
    /// Assistant message payload.
    pub message: ChatMessagePayload,
    /// Whether generation finished.
    pub done: bool,
}

/// Ollama chat message payload.
#[derive(Debug, Deserialize)]
pub struct ChatMessagePayload {
    /// Assistant text.
    pub content: String,
}

fn response_format_token(format: ResponseFormat) -> &'static str {
    match format {
        ResponseFormat::Text => "text",
        ResponseFormat::Json => "json",
    }
}
