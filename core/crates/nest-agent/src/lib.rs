//! Tool-using agent loop for the Nest framework.
//!
//! Orchestrates [`nest_ai::AiService`] completions with MCP tool execution via
//! [`nest_mcp::McpHub`].

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

mod cancel;
mod config;
mod event;
mod policy;
mod registry;
mod runner;
mod tools;

pub use cancel::CancelToken;
pub use config::{AgentConfig, AutoRunPolicy};
pub use event::AgentEvent;
pub use registry::ToolRegistry;
pub use runner::AgentLoop;
pub use tools::ToolSource;

pub use nest_error::{NestError, NestResult};

/// Maps [`nest_ai::AiError`] into [`NestError`].
pub fn ai_to_nest(error: nest_ai::AiError) -> NestError {
    NestError::network(error.message())
        .with_code(error.nest_code())
        .with_module("nest-ai")
        .with_source(error)
}
