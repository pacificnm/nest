//! JSON-RPC helpers for MCP over newline-delimited stdio.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::tool::McpToolDescriptor;

/// MCP protocol version negotiated during initialize.
pub const MCP_PROTOCOL_VERSION: &str = "2024-11-05";

/// Client name sent in the initialize handshake.
pub const NEST_MCP_CLIENT_NAME: &str = "nest-mcp";

/// Client version sent in the initialize handshake.
pub const NEST_MCP_CLIENT_VERSION: &str = "0.1.0";

/// Outbound JSON-RPC request.
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcRequest {
    /// Protocol version marker.
    pub jsonrpc: &'static str,
    /// Correlation id.
    pub id: u64,
    /// Method name.
    pub method: &'static str,
    /// Method parameters.
    pub params: Value,
}

impl JsonRpcRequest {
    /// Creates a request with the given id, method, and params.
    pub fn new(id: u64, method: &'static str, params: Value) -> Self {
        Self {
            jsonrpc: "2.0",
            id,
            method,
            params,
        }
    }
}

/// Outbound JSON-RPC notification (no id).
#[derive(Debug, Clone, Serialize)]
pub struct JsonRpcNotification {
    /// Protocol version marker.
    pub jsonrpc: &'static str,
    /// Method name.
    pub method: &'static str,
}

impl JsonRpcNotification {
    /// Creates a notification for the given method.
    pub fn new(method: &'static str) -> Self {
        Self {
            jsonrpc: "2.0",
            method,
        }
    }
}

/// Parsed inbound JSON-RPC message.
#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcMessage {
    /// Optional correlation id.
    pub id: Option<u64>,
    /// Result payload when present.
    pub result: Option<Value>,
    /// Error payload when present.
    pub error: Option<JsonRpcError>,
    /// Method name for notifications.
    pub method: Option<String>,
}

/// JSON-RPC error object.
#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcError {
    /// Error code.
    pub code: i64,
    /// Human-readable message.
    pub message: String,
}

/// Initialize request params.
pub fn initialize_params() -> Value {
    serde_json::json!({
        "protocolVersion": MCP_PROTOCOL_VERSION,
        "capabilities": {},
        "clientInfo": {
            "name": NEST_MCP_CLIENT_NAME,
            "version": NEST_MCP_CLIENT_VERSION,
        }
    })
}

/// Extracts tool list entries from a `tools/list` result.
pub fn tools_from_list_result(result: &Value) -> Vec<McpToolDescriptor> {
    result
        .get("tools")
        .and_then(Value::as_array)
        .map(|tools| {
            tools
                .iter()
                .filter_map(|tool| serde_json::from_value(tool.clone()).ok())
                .collect()
        })
        .unwrap_or_default()
}

/// Extracts plain text from a `tools/call` result.
pub fn text_from_call_result(result: &Value) -> Option<String> {
    let content = result.get("content")?.as_array()?;
    let mut parts = Vec::new();
    for block in content {
        if block.get("type").and_then(Value::as_str) == Some("text") {
            if let Some(text) = block.get("text").and_then(Value::as_str) {
                parts.push(text.to_string());
            }
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("\n"))
    }
}
