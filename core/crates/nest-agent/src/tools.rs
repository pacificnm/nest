//! Agent tool source abstraction (MCP subprocess and native in-process).

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use nest_mcp::McpHub;
use serde_json::Value;
use tokio::sync::Mutex;

use crate::tool::AgentTool;
use nest_error::NestResult;

/// Lists and invokes agent tools for the agent loop.
#[async_trait]
pub trait ToolSource: Send {
    /// Lists available tools.
    async fn list_tools(&mut self) -> NestResult<Vec<AgentTool>>;

    /// Calls a qualified tool (`server/tool`).
    async fn call_tool(&mut self, qualified_name: &str, arguments: Value) -> NestResult<String>;

    /// Whether this source supports concurrent tool calls (via shared locking).
    fn supports_parallel_calls(&self) -> bool {
        false
    }

    /// Clone handle for concurrent tool calls when [`Self::supports_parallel_calls`] is true.
    fn shared_mcp(&self) -> Option<SharedMcpHub> {
        None
    }
}

/// Thread-safe MCP hub wrapper for parallel tool calls.
#[derive(Clone)]
pub struct SharedMcpHub(Arc<Mutex<McpHub>>);

/// MCP subprocess tool source.
pub type McpToolSource = SharedMcpHub;

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
        Self::from_config_file_with_env(path, only, HashMap::new()).await
    }

    /// Loads MCP config with extra environment variables for all MCP servers.
    pub async fn from_config_file_with_env(
        path: impl AsRef<std::path::Path>,
        only: Option<&[String]>,
        extra_env: HashMap<String, String>,
    ) -> NestResult<Self> {
        McpHub::from_config_file_with_env(path, only, extra_env)
            .await
            .map(Self::new)
    }

    async fn list_tools_locked(&self) -> NestResult<Vec<AgentTool>> {
        self.0
            .lock()
            .await
            .list_tools()
            .await
            .map(|tools| tools.into_iter().map(AgentTool::from_mcp).collect())
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
    async fn list_tools(&mut self) -> NestResult<Vec<AgentTool>> {
        self.list_tools_locked().await
    }

    async fn call_tool(&mut self, qualified_name: &str, arguments: Value) -> NestResult<String> {
        self.call_tool_locked(qualified_name, arguments).await
    }

    fn supports_parallel_calls(&self) -> bool {
        true
    }

    fn shared_mcp(&self) -> Option<SharedMcpHub> {
        Some(self.clone())
    }
}

#[async_trait]
impl ToolSource for McpHub {
    async fn list_tools(&mut self) -> NestResult<Vec<AgentTool>> {
        McpHub::list_tools(self)
            .await
            .map(|tools| tools.into_iter().map(AgentTool::from_mcp).collect())
    }

    async fn call_tool(&mut self, qualified_name: &str, arguments: Value) -> NestResult<String> {
        McpHub::call_tool(self, qualified_name, arguments).await
    }
}
