//! MCP tool source abstraction.

use async_trait::async_trait;
use nest_mcp::{McpHub, McpTool};
use serde_json::Value;

use nest_error::NestResult;

/// Lists and invokes MCP tools for the agent loop.
#[async_trait]
pub trait ToolSource: Send {
    /// Lists available MCP tools.
    async fn list_tools(&mut self) -> NestResult<Vec<McpTool>>;

    /// Calls a qualified MCP tool (`server/tool`).
    async fn call_tool(&mut self, qualified_name: &str, arguments: Value) -> NestResult<String>;
}

#[async_trait]
impl ToolSource for McpHub {
    async fn list_tools(&mut self) -> NestResult<Vec<McpTool>> {
        McpHub::list_tools(self).await
    }

    async fn call_tool(&mut self, qualified_name: &str, arguments: Value) -> NestResult<String> {
        McpHub::call_tool(self, qualified_name, arguments).await
    }
}
