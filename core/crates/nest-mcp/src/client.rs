//! Single MCP server session over stdio.

use std::time::Duration;

use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::time::timeout;
use tracing::{debug, warn};

use crate::config::McpServerConfig;
use crate::protocol::{
    initialize_params, text_from_call_result, tools_from_list_result, JsonRpcMessage,
    JsonRpcNotification, JsonRpcRequest,
};
use crate::tool::{McpTool, McpToolDescriptor};
use crate::{mcp_to_nest, NestResult};

const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

/// Connected MCP server session.
pub struct McpSession {
    server_name: String,
    child: Child,
    stdin: tokio::process::ChildStdin,
    stdout: BufReader<tokio::process::ChildStdout>,
    next_id: u64,
    request_timeout: Duration,
}

impl McpSession {
    /// Spawns the server process and completes the MCP initialize handshake.
    pub async fn connect(config: McpServerConfig) -> NestResult<Self> {
        Self::connect_with_timeout(config, DEFAULT_REQUEST_TIMEOUT).await
    }

    /// Spawns the server with a custom per-request timeout.
    pub async fn connect_with_timeout(
        config: McpServerConfig,
        request_timeout: Duration,
    ) -> NestResult<Self> {
        let server_name = config.name.clone();
        debug!(server = %server_name, command = ?config.command, "spawning MCP server");

        let mut command = Command::new(&config.command);
        command
            .args(&config.args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .kill_on_drop(true);
        if let Some(cwd) = &config.cwd {
            command.current_dir(cwd);
        }
        command.envs(&config.env);

        let mut child = command.spawn().map_err(|error| {
            mcp_to_nest(format!(
                "failed to spawn MCP server {server_name} ({}): {error}",
                config.command.display()
            ))
        })?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| mcp_to_nest(format!("MCP server {server_name} stdin unavailable")))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| mcp_to_nest(format!("MCP server {server_name} stdout unavailable")))?;

        let mut session = Self {
            server_name,
            child,
            stdin,
            stdout: BufReader::new(stdout),
            next_id: 1,
            request_timeout,
        };

        session.initialize().await?;
        Ok(session)
    }

    /// Server id for this session.
    pub fn server_name(&self) -> &str {
        &self.server_name
    }

    /// Lists tools advertised by the server.
    pub async fn list_tools(&mut self) -> NestResult<Vec<McpToolDescriptor>> {
        let result = self.request("tools/list", json!({})).await?;
        Ok(tools_from_list_result(&result))
    }

    /// Calls a tool by server-local name.
    pub async fn call_tool(&mut self, name: &str, arguments: Value) -> NestResult<String> {
        let result = self
            .request(
                "tools/call",
                json!({
                    "name": name,
                    "arguments": arguments,
                }),
            )
            .await?;
        text_from_call_result(&result).ok_or_else(|| {
            mcp_to_nest(format!(
                "MCP tool {}/{} returned no text content",
                self.server_name, name
            ))
        })
    }

    /// Gracefully shuts down the child process.
    pub async fn shutdown(mut self) -> NestResult<()> {
        let _ = self.child.kill().await;
        let _ = self.child.wait().await;
        Ok(())
    }

    async fn initialize(&mut self) -> NestResult<()> {
        let _ = self.request("initialize", initialize_params()).await?;
        self.notify("notifications/initialized").await
    }

    async fn notify(&mut self, method: &'static str) -> NestResult<()> {
        let notification = JsonRpcNotification::new(method);
        let line = serde_json::to_string(&notification)
            .map_err(|error| mcp_to_nest(format!("failed to encode MCP notification: {error}")))?;
        self.stdin
            .write_all(line.as_bytes())
            .await
            .map_err(|error| mcp_to_nest(format!("MCP write failed: {error}")))?;
        self.stdin
            .write_all(b"\n")
            .await
            .map_err(|error| mcp_to_nest(format!("MCP write failed: {error}")))?;
        self.stdin
            .flush()
            .await
            .map_err(|error| mcp_to_nest(format!("MCP flush failed: {error}")))?;
        Ok(())
    }

    async fn request(&mut self, method: &'static str, params: Value) -> NestResult<Value> {
        let id = self.next_id;
        self.next_id += 1;
        let request = JsonRpcRequest::new(id, method, params);
        let line = serde_json::to_string(&request)
            .map_err(|error| mcp_to_nest(format!("failed to encode MCP request: {error}")))?;
        self.stdin
            .write_all(line.as_bytes())
            .await
            .map_err(|error| mcp_to_nest(format!("MCP write failed: {error}")))?;
        self.stdin
            .write_all(b"\n")
            .await
            .map_err(|error| mcp_to_nest(format!("MCP write failed: {error}")))?;
        self.stdin
            .flush()
            .await
            .map_err(|error| mcp_to_nest(format!("MCP flush failed: {error}")))?;

        self.read_response(id).await
    }

    async fn read_response(&mut self, id: u64) -> NestResult<Value> {
        let server = self.server_name.clone();
        let timeout_duration = self.request_timeout;
        let read_loop = async {
            loop {
                let mut line = String::new();
                let bytes = self
                    .stdout
                    .read_line(&mut line)
                    .await
                    .map_err(|error| mcp_to_nest(format!("MCP read failed: {error}")))?;
                if bytes == 0 {
                    return Err(mcp_to_nest(format!(
                        "MCP server {server} closed stdout while waiting for response"
                    )));
                }
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let message: JsonRpcMessage = serde_json::from_str(trimmed).map_err(|error| {
                    mcp_to_nest(format!("MCP server {server} sent invalid JSON: {error}"))
                })?;
                if message.id != Some(id) {
                    if message.method.is_some() {
                        debug!(server = %server, method = ?message.method, "ignored MCP notification");
                    } else {
                        warn!(server = %server, expected = id, got = ?message.id, "skipped unmatched MCP response");
                    }
                    continue;
                }
                if let Some(error) = message.error {
                    return Err(mcp_to_nest(format!(
                        "MCP server {server} error {}: {}",
                        error.code, error.message
                    )));
                }
                return message.result.ok_or_else(|| {
                    mcp_to_nest(format!("MCP server {server} response missing result"))
                });
            }
        };

        timeout(timeout_duration, read_loop).await.map_err(|_| {
            mcp_to_nest(format!(
                "MCP server {server} timed out after {}s",
                timeout_duration.as_secs()
            ))
        })?
    }
}

impl McpSession {
    /// Lists tools as hub entries for this server.
    pub async fn list_hub_tools(&mut self) -> NestResult<Vec<McpTool>> {
        Ok(self
            .list_tools()
            .await?
            .into_iter()
            .map(|descriptor| McpTool::from_descriptor(self.server_name.clone(), descriptor))
            .collect())
    }
}
