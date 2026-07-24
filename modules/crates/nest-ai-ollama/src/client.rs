//! Low-level Ollama HTTP client.

#![allow(clippy::result_large_err)]

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
        let config = self.config.snapshot();
        let model = request.model.as_deref().unwrap_or(&config.model).to_string();
        let body = build_request_body(&model, request, false, &config)?;

        let url = format!("{}/api/chat", config.base_url);
        debug!(model = %model, tools = request.tools.len(), "ollama chat request");
        self.http
            .post_json(&url, &body)
            .await
            .map_err(OllamaError::from)
    }

    /// Calls `POST /api/chat` with streaming enabled.
    pub async fn chat_stream(&self, request: &CompletionRequest) -> OllamaResult<ChatStream> {
        let config = self.config.snapshot();
        let model = request.model.as_deref().unwrap_or(&config.model).to_string();
        let body = build_request_body(&model, request, true, &config)?;

        let url = format!("{}/api/chat", config.base_url);
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
    config: &OllamaConfig,
) -> OllamaResult<ChatRequestBody> {
    let tools = if request.tools.is_empty() {
        None
    } else {
        Some(to_ollama_tools(&request.tools))
    };
    let options = if config.num_ctx.is_some() || config.temperature.is_some() {
        Some(OllamaOptions {
            num_ctx: config.num_ctx,
            temperature: config.temperature,
        })
    } else {
        None
    };
    Ok(ChatRequestBody {
        model: model.to_string(),
        messages: to_ollama_messages(&request.messages),
        stream,
        format: request.format.map(response_format_token),
        tools,
        options,
        think: config.think.then_some(true),
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
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    think: Option<bool>,
}

/// Model runtime parameters sent as Ollama's `options` object.
#[derive(Debug, Serialize)]
struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    num_ctx: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
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
    #[serde(default)]
    load_duration: u64,
    #[serde(default)]
    prompt_eval_count: u32,
    #[serde(default)]
    prompt_eval_duration: u64,
    #[serde(default)]
    eval_count: u32,
    #[serde(default)]
    eval_duration: u64,
    #[serde(default)]
    total_duration: u64,
}

impl ChatResponse {
    /// Token and timing stats when Ollama includes them on the final response.
    pub fn metrics(&self) -> Option<nest_ai::CompletionMetrics> {
        self.done.then(|| {
            nest_ai::CompletionMetrics::from_timing(
                self.prompt_eval_count,
                self.eval_count,
                self.load_duration,
                self.prompt_eval_duration,
                self.eval_duration,
                self.total_duration,
            )
        })
    }
}

fn response_format_token(format: ResponseFormat) -> &'static str {
    match format {
        ResponseFormat::Text => "text",
        ResponseFormat::Json => "json",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nest_http_client::HttpClientConfig;
    use serde_json::Value;
    use std::sync::{Arc, Mutex};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn client_with_config(config: OllamaConfig) -> OllamaClient {
        let http = HttpClientService::new(HttpClientConfig::default()).unwrap();
        OllamaClient::new(http, OllamaSharedConfig::new(config)).unwrap()
    }

    #[tokio::test]
    async fn chat_sends_options_and_think_when_configured() {
        let server = MockServer::start().await;
        let captured: Arc<Mutex<Option<Value>>> = Arc::new(Mutex::new(None));
        let captured_clone = captured.clone();

        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(move |request: &wiremock::Request| {
                let body: Value = serde_json::from_slice(&request.body).unwrap();
                *captured_clone.lock().unwrap() = Some(body);
                ResponseTemplate::new(200).set_body_string(
                    "{\"model\":\"qwen3:32b-q4_K_M\",\"message\":{\"role\":\"assistant\",\"content\":\"ok\"},\"done\":true}\n",
                )
            })
            .mount(&server)
            .await;

        let config = OllamaConfig::new(server.uri(), "qwen3:32b-q4_K_M")
            .with_num_ctx(40960)
            .with_temperature(0.2)
            .with_think(true);
        let client = client_with_config(config);

        client
            .chat(&CompletionRequest::user_message("hi"))
            .await
            .unwrap();

        let body = captured.lock().unwrap().clone().unwrap();
        assert_eq!(body["options"]["num_ctx"], 40960);
        assert_eq!(body["options"]["temperature"], 0.2);
        assert_eq!(body["think"], true);
    }

    #[tokio::test]
    async fn chat_omits_options_and_think_when_not_configured() {
        let server = MockServer::start().await;
        let captured: Arc<Mutex<Option<Value>>> = Arc::new(Mutex::new(None));
        let captured_clone = captured.clone();

        Mock::given(method("POST"))
            .and(path("/api/chat"))
            .respond_with(move |request: &wiremock::Request| {
                let body: Value = serde_json::from_slice(&request.body).unwrap();
                *captured_clone.lock().unwrap() = Some(body);
                ResponseTemplate::new(200).set_body_string(
                    "{\"model\":\"smollm2:360m\",\"message\":{\"role\":\"assistant\",\"content\":\"ok\"},\"done\":true}\n",
                )
            })
            .mount(&server)
            .await;

        let config = OllamaConfig::new(server.uri(), "smollm2:360m");
        let client = client_with_config(config);

        client
            .chat(&CompletionRequest::user_message("hi"))
            .await
            .unwrap();

        let body = captured.lock().unwrap().clone().unwrap();
        assert!(body.get("options").is_none());
        assert!(body.get("think").is_none());
    }
}
