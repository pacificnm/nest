//! Combines MCP tools with native in-process file tools.

use async_trait::async_trait;
use nest_mcp::split_qualified_tool_name;
use serde_json::Value;

use crate::file_tools::{FileToolSource, FILE_SERVER};
use crate::tool::AgentTool;
use crate::tools::{SharedMcpHub, ToolSource};
use crate::{NestError, NestResult};

/// MCP tools plus optional native [`FileToolSource`].
pub struct CompositeToolSource {
    mcp: SharedMcpHub,
    file: Option<FileToolSource>,
}

impl CompositeToolSource {
    /// Creates a composite source with MCP tools only.
    pub fn new(mcp: SharedMcpHub) -> Self {
        Self { mcp, file: None }
    }

    /// Adds native workspace file tools backed by [`nest_file::FileService`].
    pub fn with_files(mut self, files: nest_file::FileService) -> Self {
        self.file = Some(FileToolSource::new(files));
        self
    }
}

#[async_trait]
impl ToolSource for CompositeToolSource {
    async fn list_tools(&mut self) -> NestResult<Vec<AgentTool>> {
        let mut tools = self.mcp.list_tools().await?;
        if let Some(file) = &mut self.file {
            tools.extend(file.list_tools().await?);
        }
        Ok(tools)
    }

    async fn call_tool(&mut self, qualified_name: &str, arguments: Value) -> NestResult<String> {
        let (server, _) = split_qualified_tool_name(qualified_name)?;
        if server == FILE_SERVER {
            let file = self.file.as_mut().ok_or_else(|| {
                NestError::network("file tools are not configured for this agent run")
                    .with_module("nest-agent")
            })?;
            return file.call_tool(qualified_name, arguments).await;
        }
        self.mcp.call_tool(qualified_name, arguments).await
    }

    fn supports_parallel_calls(&self) -> bool {
        // File tools run synchronously in-process; keep tool calls sequential.
        false
    }

    fn shared_mcp(&self) -> Option<SharedMcpHub> {
        None
    }
}
