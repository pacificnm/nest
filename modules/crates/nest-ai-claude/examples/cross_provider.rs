//! Runs the *same* [`CompletionRequest`] against both `nest-ai-ollama` and
//! `nest-ai-claude`, asserting a valid [`CompletionResponse`] from each -
//! the actual proof that swapping [`AiProvider`] implementations works, not
//! just that both crates compile independently.
//!
//! Each provider is pointed at its own `wiremock` mock server rather than a
//! live backend, so this example runs fully offline with no API key or
//! local Ollama install required. Run with:
//!
//! ```bash
//! cargo run --example cross_provider -p nest-ai-claude
//! ```

use nest_ai::{AiProvider, CompletionRequest, CompletionResponse};
use nest_ai_claude::ClaudeAiProvider;
use nest_ai_ollama::{OllamaConfig, OllamaProvider};
use nest_claude::ClaudeConfig;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let request = CompletionRequest::user_message("What is the capital of France?");

    let ollama_response = run_ollama(request.clone()).await?;
    println!(
        "[{}] {}",
        ollama_response.model.as_str(),
        ollama_response.content
    );
    assert!(!ollama_response.content.is_empty());

    let claude_response = run_claude(request).await?;
    println!(
        "[{}] {}",
        claude_response.model.as_str(),
        claude_response.content
    );
    assert!(!claude_response.content.is_empty());

    println!(
        "\nBoth providers answered the same CompletionRequest through the same AiProvider trait."
    );
    Ok(())
}

async fn run_ollama(
    request: CompletionRequest,
) -> Result<CompletionResponse, Box<dyn std::error::Error>> {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/chat"))
        .respond_with(ResponseTemplate::new(200).set_body_string(
            "{\"model\":\"smollm2:360m\",\"message\":{\"role\":\"assistant\",\"content\":\"Paris.\"},\"done\":true}\n",
        ))
        .mount(&server)
        .await;

    let config = OllamaConfig::new(server.uri(), "smollm2:360m");
    let provider = OllamaProvider::new(config)?;
    Ok(provider.complete(request).await?)
}

async fn run_claude(
    request: CompletionRequest,
) -> Result<CompletionResponse, Box<dyn std::error::Error>> {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/messages"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "msg_1",
            "type": "message",
            "role": "assistant",
            "content": [{"type": "text", "text": "Paris."}],
            "model": "claude-opus-4-8",
            "stop_reason": "end_turn",
            "stop_sequence": null,
            "usage": {"input_tokens": 10, "output_tokens": 5}
        })))
        .mount(&server)
        .await;

    let config = ClaudeConfig::builder()
        .api_key("test-key")
        .base_url(server.uri())
        .build()?;
    let provider = ClaudeAiProvider::new(config)?;
    Ok(provider.complete(request).await?)
}
