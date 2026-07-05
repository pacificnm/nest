//! Agent-visible tool descriptors (native and MCP-backed).

use nest_mcp::{qualify_tool_name, McpTool};
use serde_json::Value;

/// Whether a tool runs in-process or via an MCP subprocess.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolOrigin {
    /// In-process Rust implementation.
    Native,
    /// MCP subprocess server.
    Mcp,
}

/// Tool metadata exposed to the model and UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentTool {
    /// How the tool is implemented.
    pub origin: ToolOrigin,
    /// Server or module id (`nest-memory`, `nest-file`, …).
    pub server: String,
    /// Tool name on the server.
    pub name: String,
    /// Qualified name (`server/tool`).
    pub qualified_name: String,
    /// Human-readable description.
    pub description: String,
    /// JSON Schema for arguments.
    pub input_schema: Value,
}

impl AgentTool {
    /// Converts an MCP hub tool into an agent tool.
    pub fn from_mcp(mcp: McpTool) -> Self {
        Self {
            origin: ToolOrigin::Mcp,
            server: mcp.server,
            name: mcp.name,
            qualified_name: mcp.qualified_name,
            description: mcp.description,
            input_schema: mcp.input_schema,
        }
    }

    /// Builds a native in-process tool descriptor.
    pub fn native(
        server: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: Value,
    ) -> Self {
        let server = server.into();
        let name = name.into();
        Self {
            origin: ToolOrigin::Native,
            qualified_name: qualify_tool_name(&server, &name),
            server,
            name,
            description: description.into(),
            input_schema,
        }
    }

    /// Stable model-visible function name (`nest_memory__search_project_memory`).
    pub fn model_name(&self) -> String {
        crate::registry::model_tool_name(&self.server, &self.name)
    }
}
