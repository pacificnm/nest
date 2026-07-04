//! Multi-server MCP hub.

use std::collections::HashMap;
use std::path::Path;

use serde_json::Value;
use tracing::debug;

use crate::client::McpSession;
use crate::config::{load_mcp_config, McpServerConfig};
use crate::tool::McpTool;
use crate::{split_qualified_tool_name, NestResult};

/// Manages multiple MCP server sessions.
pub struct McpHub {
    sessions: HashMap<String, McpSession>,
}

impl McpHub {
    /// Connects to all provided server configs.
    pub async fn connect_all(configs: Vec<McpServerConfig>) -> NestResult<Self> {
        let mut sessions = HashMap::new();
        for config in configs {
            let name = config.name.clone();
            let session = McpSession::connect(config).await?;
            sessions.insert(name, session);
        }
        Ok(Self { sessions })
    }

    /// Loads MCP config from disk and connects to selected servers.
    pub async fn from_config_file(
        path: impl AsRef<Path>,
        only: Option<&[String]>,
    ) -> NestResult<Self> {
        let path = path.as_ref();
        let base_dir = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let config = load_mcp_config(path)?;
        let servers = config.servers(base_dir, only)?;
        Self::connect_all(servers).await
    }

    /// Returns configs from a parsed file without connecting (for tests/planning).
    pub fn server_configs_from_file(
        path: impl AsRef<Path>,
        only: Option<&[String]>,
    ) -> NestResult<Vec<McpServerConfig>> {
        let path = path.as_ref();
        let base_dir = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        load_mcp_config(path)?.servers(base_dir, only)
    }

    /// Connected server ids.
    pub fn server_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.sessions.keys().cloned().collect();
        names.sort();
        names
    }

    /// Lists tools from all connected servers.
    pub async fn list_tools(&mut self) -> NestResult<Vec<McpTool>> {
        let mut tools = Vec::new();
        for name in self.server_names() {
            let session = self
                .sessions
                .get_mut(&name)
                .expect("server name collected from keys");
            tools.extend(session.list_hub_tools().await?);
        }
        Ok(tools)
    }

    /// Calls a qualified tool (`server/tool`).
    pub async fn call_tool(&mut self, qualified_name: &str, arguments: Value) -> NestResult<String> {
        let (server, tool) = split_qualified_tool_name(qualified_name)?;
        let session = self.sessions.get_mut(server).ok_or_else(|| {
            crate::mcp_to_nest(format!("MCP server not connected: {server}"))
        })?;
        debug!(server, tool, "calling MCP tool");
        session.call_tool(tool, arguments).await
    }

    /// Shuts down all server processes.
    pub async fn shutdown(self) -> NestResult<()> {
        for (name, session) in self.sessions {
            debug!(server = %name, "shutting down MCP server");
            session.shutdown().await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[tokio::test]
    async fn mock_stdio_server_lists_and_calls_tool() {
        let script = r#"#!/usr/bin/env python3
import json, sys
for line in sys.stdin:
    msg = json.loads(line)
    mid = msg.get("id")
    method = msg.get("method")
    if method == "initialize":
        out = {"jsonrpc":"2.0","id":mid,"result":{"protocolVersion":"2024-11-05","capabilities":{},"serverInfo":{"name":"mock","version":"1"}}}
    elif method == "notifications/initialized":
        continue
    elif method == "tools/list":
        out = {"jsonrpc":"2.0","id":mid,"result":{"tools":[{"name":"echo","description":"echo","inputSchema":{"type":"object"}}]}}
    elif method == "tools/call":
        out = {"jsonrpc":"2.0","id":mid,"result":{"content":[{"type":"text","text":"pong"}]}}
    else:
        out = {"jsonrpc":"2.0","id":mid,"error":{"code":-32601,"message":"unknown"}}
    sys.stdout.write(json.dumps(out) + "\n")
    sys.stdout.flush()
"#;
        let temp = std::env::temp_dir().join(format!("nest-mcp-mock-{}.py", std::process::id()));
        std::fs::write(&temp, script).unwrap();

        let config = McpServerConfig {
            name: "mock".into(),
            command: PathBuf::from("python3"),
            args: vec![temp.to_string_lossy().into_owned()],
            cwd: None,
            env: HashMap::new(),
        };

        let mut hub = McpHub::connect_all(vec![config]).await.unwrap();
        let tools = hub.list_tools().await.unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].qualified_name, "mock/echo");

        let result = hub
            .call_tool("mock/echo", serde_json::json!({}))
            .await
            .unwrap();
        assert_eq!(result, "pong");
        hub.shutdown().await.unwrap();
        let _ = std::fs::remove_file(temp);
    }
}
