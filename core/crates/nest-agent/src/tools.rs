//! MCP tool source abstraction.

use std::sync::Arc;

use async_trait::async_trait;
use nest_mcp::{McpHub, McpTool};
use serde_json::Value;
use tokio::sync::Mutex;

use nest_error::NestResult;

/// Lists and invokes MCP tools for the agent loop.
#[async_trait]
pub trait ToolSource: Send {
    /// Lists available MCP tools.
    async fn list_tools(&mut self) -> NestResult<Vec<McpTool>>;

    /// Calls a qualified MCP tool (`server/tool`).
    async fn call_tool(&mut self, qualified_name: &str, arguments: Value) -> NestResult<String>;

    /// Whether this source supports concurrent tool calls (via shared locking).
    fn supports_parallel_calls(&self) -> bool {
        false
    }

    /// Clone handle for concurrent tool calls when [`Self::supports_parallel_calls`] is true.
    fn shared(&self) -> Option<SharedMcpHub> {
        None
    }
}

/// Thread-safe MCP hub wrapper for parallel tool calls.
#[derive(Clone)]
pub struct SharedMcpHub(Arc<Mutex<McpHub>>);

impl SharedMcpHub {
    /// Wraps a connected hub for concurrent access.
    pub fn new(hub: McpHub) -> Self {
        Self(Arc::new(Mutex::new(hub)))
    }

    /// Loads MCP config and connects to selected servers.
    pub async fn from_config_file(
        path: impl AsRef<std::path::Path>,
        only: Option<&[String]>,
    ) -> NestResult<Self> {
        McpHub::from_config_file(path, only)
            .await
            .map(Self::new)
    }

    async fn list_tools_locked(&self) -> NestResult<Vec<McpTool>> {
        self.0.lock().await.list_tools().await
    }

    async fn call_tool_locked(
        &self,
        qualified_name: &str,
        arguments: Value,
    ) -> NestResult<String> {
        self.0
            .lock()
            .await
            .call_tool(qualified_name, arguments)
            .await
    }
}

#[async_trait]
impl ToolSource for SharedMcpHub {
    async fn list_tools(&mut self) -> NestResult<Vec<McpTool>> {
        self.list_tools_locked().await
    }

    async fn call_tool(&mut self, qualified_name: &str, arguments: Value) -> NestResult<String> {
        self.call_tool_locked(qualified_name, arguments).await
    }

    fn supports_parallel_calls(&self) -> bool {
        true
    }

    fn shared(&self) -> Option<SharedMcpHub> {
        Some(self.clone())
    }
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
