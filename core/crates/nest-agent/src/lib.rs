//! Tool-using agent loop for the Nest framework.
//!
//! Orchestrates [`nest_ai::AiService`] completions with MCP tool execution via
//! [`nest_mcp::McpHub`].

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

mod cancel;
mod catalog;
mod composite;
mod config;
mod event;
mod file_ops;
mod file_tools;
mod policy;
mod registry;
mod runner;
mod tool;
mod tools;
mod validation;
mod workspace_ops;

pub use cancel::CancelToken;
pub use catalog::probe_tools;
pub use composite::CompositeToolSource;
pub use config::{AgentConfig, AutoRunPolicy};
pub use event::AgentEvent;
pub use file_ops::{
    create_directory, create_file, ensure_directory, join_rel_path,
    write_file as write_workspace_file,
};
pub use file_tools::{FileToolSource, FILE_SERVER};
pub use registry::ToolRegistry;
pub use runner::AgentLoop;
pub use tool::{AgentTool, ToolOrigin};
pub use tools::{McpToolSource, SharedMcpHub, ToolSource};
pub use validation::{looks_like_schema_arguments, parse_tool_calls_from_content, validate_tool_arguments};

pub use nest_error::{NestError, NestResult};

/// Maps [`nest_ai::AiError`] into [`NestError`].
pub fn ai_to_nest(error: nest_ai::AiError) -> NestError {
    NestError::network(error.message())
        .with_code(error.nest_code())
        .with_module("nest-ai")
        .with_source(error)
}
