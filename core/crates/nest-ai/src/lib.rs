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
pub use provider::AiProvider;
pub use service::AiService;
pub use stream::{CompletionChunk, CompletionStream};
pub use tools::{merge_tool_calls, ToolCall, ToolDefinition};
pub use types::{ChatMessage, ChatRole, CompletionRequest, CompletionResponse, ResponseFormat};
