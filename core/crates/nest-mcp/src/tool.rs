//! MCP tool descriptors.

use serde::Deserialize;
use serde_json::Value;

/// Tool metadata returned by `tools/list` (server-local name).
#[derive(Debug, Clone, Deserialize)]
pub struct McpToolDescriptor {
    /// Tool name on the server.
    pub name: String,
    /// Human-readable description.
    #[serde(default)]
    pub description: String,
    /// JSON Schema for arguments.
    #[serde(rename = "inputSchema", default)]
    pub input_schema: Value,
}

/// Tool exposed through [`crate::McpHub`] with server qualification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpTool {
    /// MCP server id.
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

impl McpTool {
    /// Builds a hub tool from server id and list descriptor.
    pub fn from_descriptor(server: impl Into<String>, descriptor: McpToolDescriptor) -> Self {
        let server = server.into();
        let qualified_name = crate::qualify_tool_name(&server, &descriptor.name);
        Self {
            server,
            name: descriptor.name,
            qualified_name,
            description: descriptor.description,
            input_schema: descriptor.input_schema,
        }
    }
}
