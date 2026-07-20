//! AI inference provider contracts for the Nest framework.
//!
//! nest-ai defines **what** inference means. Provider crates (`nest-ai-ollama`,
//! future OpenAI/Gemini adapters) decide **how** HTTP calls happen.
//!
//! Apps depend on [`AiProvider`], not a specific engine:
//!
//! ```ignore
//! use std::sync::Arc;
//! use nest_ai::{AiProvider, CompletionRequest};
//! use nest_ai_ollama::OllamaProvider;
//!
//! let ai: Arc<dyn AiProvider> = Arc::new(OllamaProvider::new(config)?);
//! let response = ai.complete(CompletionRequest::user_message("Hello")).await?;
//! ```

#![deny(missing_docs)]

pub mod codes;
pub mod error;
pub mod metrics;
pub mod prelude;
pub mod provider;
pub mod service;
pub mod stream;
pub mod tools;
pub mod types;

pub use error::{AiError, AiErrorKind, AiResult};
pub use metrics::CompletionMetrics;
use nest_error::NestErrorKind;
pub use provider::AiProvider;
pub use service::AiService;
pub use stream::{CompletionChunk, CompletionStream};
pub use tools::{merge_tool_calls, ToolCall, ToolDefinition};
pub use types::{ChatMessage, ChatRole, CompletionRequest, CompletionResponse, ResponseFormat};

pub use nest_error::{NestError, NestResult};

impl From<AiError> for NestError {
    fn from(error: AiError) -> Self {
        let kind = match error.kind() {
            AiErrorKind::InvalidInput => NestErrorKind::Validation,
            AiErrorKind::Request => NestErrorKind::Network,
            AiErrorKind::Parse => NestErrorKind::Data,
            AiErrorKind::Config => NestErrorKind::Config,
        };

        NestError::new(kind, error.message())
            .with_code(error.nest_code())
            .with_module("nest-ai")
            .with_source(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nest_error::NestErrorKind;

    #[test]
    fn ai_error_converts_to_nest_error() {
        let err = AiError::request("upstream unavailable");
        let nest_error: NestError = err.into();
        assert_eq!(nest_error.kind(), NestErrorKind::Network);
        assert_eq!(nest_error.code(), Some(codes::NEST_AI_REQUEST_FAILED));
    }
}
