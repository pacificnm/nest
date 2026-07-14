//! Claude (Anthropic) [`nest_ai::AiProvider`] adapter for the Nest framework.
//!
//! Wraps [`nest_claude::ClaudeClient`] behind the provider-agnostic
//! [`nest_ai::AiProvider`] trait, so applications can swap between Claude and
//! other providers (e.g. `nest-ai-ollama`) without changing call sites.

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

pub mod error;
pub mod message;
pub mod module;
pub mod provider;

pub use error::claude_to_ai_error;
pub use message::{to_claude_messages, to_claude_tools};
pub use module::{ClaudeAiModule, CLAUDE_AI_MODULE_ID};
pub use provider::ClaudeAiProvider;
