//! Multi-server MCP hub.

use std::collections::HashMap;
use std::path::Path;

use serde_json::Value;
use tracing::{debug, warn};

use crate::client::McpSession;
use crate::config::{load_mcp_config, McpServerConfig};
use crate::tool::McpTool;
use crate::{split_qualified_tool_name, NestResult};

/// Manages multiple MCP server sessions.
pub struct McpHub {
    sessions: HashMap<String, McpSession>,
    configs: HashMap<String, McpServerConfig>,
}

impl McpHub {
    /// Connects to all provided server configs.
    pub async fn connect_all(configs: Vec<McpServerConfig>) -> NestResult<Self> {
        let mut sessions = HashMap::new();
        let mut stored = HashMap::new();
        for config in configs {
            let name = config.name.clone();
            stored.insert(name.clone(), config.clone());
            let session = McpSession::connect(config).await?;
            sessions.insert(name, session);
        }
        Ok(Self {
            sessions,
            configs: stored,
        })
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
            match session.list_hub_tools().await {
                Ok(server_tools) => tools.extend(server_tools),
                Err(error) if is_reconnect_error(&error) => {
                    warn!(server = %name, "MCP list_tools failed; reconnecting");
                    self.reconnect_server(&name).await?;
                    let session = self.sessions.get_mut(&name).expect("reconnected");
                    tools.extend(session.list_hub_tools().await?);
                }
                Err(error) => return Err(error),
            }
        }
        Ok(tools)
    }

    /// Calls a qualified tool (`server/tool`), reconnecting once on transport failure.
    pub async fn call_tool(&mut self, qualified_name: &str, arguments: Value) -> NestResult<String> {
        match self.call_tool_inner(qualified_name, arguments.clone()).await {
            Ok(result) => Ok(result),
            Err(error) if is_reconnect_error(&error) => {
                let (server, tool) = split_qualified_tool_name(qualified_name)?;
                warn!(server, tool, "MCP tool call failed; reconnecting server");
                self.reconnect_server(server).await?;
                self.call_tool_inner(qualified_name, arguments).await
            }
            Err(error) => Err(error),
        }
    }

    /// Shuts down all server processes.
    pub async fn shutdown(self) -> NestResult<()> {
        for (name, session) in self.sessions {
            debug!(server = %name, "shutting down MCP server");
            session.shutdown().await?;
        }
        Ok(())
    }

    async fn call_tool_inner(
        &mut self,
        qualified_name: &str,
        arguments: Value,
    ) -> NestResult<String> {
        let (server, tool) = split_qualified_tool_name(qualified_name)?;
        let session = self.sessions.get_mut(server).ok_or_else(|| {
            crate::mcp_to_nest(format!("MCP server not connected: {server}"))
        })?;
        debug!(server, tool, "calling MCP tool");
        session.call_tool(tool, arguments).await
    }

    async fn reconnect_server(&mut self, server: &str) -> NestResult<()> {
        let config = self.configs.get(server).cloned().ok_or_else(|| {
            crate::mcp_to_nest(format!("no launch config for MCP server: {server}"))
        })?;
        if let Some(session) = self.sessions.remove(server) {
            let _ = session.shutdown().await;
        }
        let session = McpSession::connect(config).await?;
        self.sessions.insert(server.to_string(), session);
        Ok(())
    }
}

fn is_reconnect_error(error: &nest_error::NestError) -> bool {
    let message = error.to_string().to_ascii_lowercase();
    message.contains("closed stdout")
        || message.contains("read failed")
        || message.contains("write failed")
        || message.contains("timed out")
        || message.contains("mcp read failed")
        || message.contains("mcp write failed")
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

    #[test]
    fn reconnect_error_detection() {
        let error = crate::mcp_to_nest("MCP server nest-memory closed stdout while waiting");
        assert!(is_reconnect_error(&error));
    }
}
