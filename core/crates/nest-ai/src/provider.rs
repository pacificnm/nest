//! AI provider contract.

use async_trait::async_trait;

use crate::error::{AiError, AiResult};
use crate::stream::CompletionStream;
use crate::types::{CompletionRequest, CompletionResponse};

/// Completes chat-style prompts against an inference backend.
#[async_trait]
pub trait AiProvider: Send + Sync {
    /// Stable provider id (`ollama`, `openai`, …).
    fn provider_id(&self) -> &'static str;

    /// Runs a completion request and returns assistant text.
    async fn complete(&self, request: CompletionRequest) -> AiResult<CompletionResponse>;

    /// Streams incremental completion chunks when supported by the provider.
    async fn stream_complete(
        &self,
        _request: CompletionRequest,
    ) -> AiResult<CompletionStream> {
        Err(AiError::invalid_input(
            "streaming is not supported by this provider",
        ))
    }
}
