//! MCP stdio client for the Nest framework.
//!
//! Spawns MCP server child processes, performs the initialize handshake, and
//! exposes [`McpHub`] for listing and calling tools.

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

mod client;
mod config;
mod hub;
mod protocol;
mod tool;

pub use client::McpSession;
pub use config::{load_mcp_config, McpConfigFile, McpServerConfig};
pub use hub::McpHub;
pub use protocol::{MCP_PROTOCOL_VERSION, NEST_MCP_CLIENT_NAME, NEST_MCP_CLIENT_VERSION};
pub use tool::McpTool;

pub use nest_error::{NestError, NestResult};

/// Converts an internal MCP failure into a [`NestError`].
pub fn mcp_to_nest(message: impl Into<String>) -> NestError {
    NestError::network(message)
        .with_code("NEST_MCP_ERROR")
        .with_module("nest-mcp")
}

/// Qualifies a tool name with its server id (`nest-memory/search_project_memory`).
pub fn qualify_tool_name(server: &str, tool: &str) -> String {
    format!("{server}/{tool}")
}

/// Splits a qualified tool name into `(server, tool)`.
pub fn split_qualified_tool_name(qualified: &str) -> NestResult<(&str, &str)> {
    let (server, tool) = qualified
        .split_once('/')
        .ok_or_else(|| mcp_to_nest(format!("invalid qualified tool name: {qualified}")))?;
    if server.is_empty() || tool.is_empty() {
        return Err(mcp_to_nest(format!(
            "invalid qualified tool name: {qualified}"
        )));
    }
    Ok((server, tool))
}
