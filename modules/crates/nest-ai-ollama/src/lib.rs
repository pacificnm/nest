//! Ollama inference adapter for nest-ai.
//!
//! Implements [`nest_ai::AiProvider`] against the local Ollama HTTP API
//! (`POST /api/chat`).

#![deny(missing_docs)]

pub mod client;
pub mod codes;
pub mod config;
pub mod error;
pub mod message;
pub mod module;
pub mod provider;
pub mod shared;
pub mod stream;

pub use config::{AiSection, OllamaConfig, DEFAULT_BASE_URL, DEFAULT_MODEL, DEFAULT_PORT};
pub use error::{OllamaError, OllamaResult};
pub use module::{OllamaModule, OLLAMA_MODULE_ID};
pub use provider::{is_available, OllamaProvider};
pub use shared::OllamaSharedConfig;

pub use nest_ai::{AiProvider, CompletionRequest, CompletionResponse};
