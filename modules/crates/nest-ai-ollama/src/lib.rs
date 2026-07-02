//! Ollama inference adapter for nest-ai.
//!
//! Implements [`nest_ai::AiProvider`] against the local Ollama HTTP API
//! (`POST /api/chat`).

#![deny(missing_docs)]

pub mod client;
pub mod codes;
pub mod config;
pub mod error;
pub mod provider;

pub use config::{OllamaConfig, DEFAULT_BASE_URL, DEFAULT_MODEL};
pub use error::{OllamaError, OllamaResult};
pub use provider::{is_available, OllamaProvider};

pub use nest_ai::{AiProvider, CompletionRequest, CompletionResponse};
