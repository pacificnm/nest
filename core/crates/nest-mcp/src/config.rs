//! Cursor-compatible MCP configuration loading.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::{mcp_to_nest, NestResult};

/// Parsed `.cursor/mcp.json` document.
#[derive(Debug, Clone, Deserialize)]
pub struct McpConfigFile {
    /// Server name → launch config.
    #[serde(rename = "mcpServers", default)]
    pub mcp_servers: HashMap<String, McpServerEntry>,
}

/// Raw server entry from JSON (Cursor shape).
#[derive(Debug, Clone, Deserialize)]
pub struct McpServerEntry {
    /// Executable path.
    pub command: PathBuf,
    /// Arguments passed to the executable.
    #[serde(default)]
    pub args: Vec<String>,
    /// Working directory for the child process.
    pub cwd: Option<PathBuf>,
    /// Extra environment variables.
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// Resolved launch configuration for one MCP server.
#[derive(Debug, Clone)]
pub struct McpServerConfig {
    /// Stable server id (`nest-memory`).
    pub name: String,
    /// Executable path.
    pub command: PathBuf,
    /// Arguments passed to the executable.
    pub args: Vec<String>,
    /// Working directory for the child process.
    pub cwd: Option<PathBuf>,
    /// Extra environment variables.
    pub env: HashMap<String, String>,
}

impl McpConfigFile {
    /// Returns server configs, optionally filtered by name.
    pub fn servers(
        &self,
        base_dir: &Path,
        only: Option<&[String]>,
    ) -> NestResult<Vec<McpServerConfig>> {
        let mut configs = Vec::new();
        for (name, entry) in &self.mcp_servers {
            if let Some(filter) = only {
                if !filter.iter().any(|item| item == name) {
                    continue;
                }
            }
            configs.push(entry.to_config(name, base_dir)?);
        }
        configs.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(configs)
    }
}

impl McpServerEntry {
    fn to_config(&self, name: &str, base_dir: &Path) -> NestResult<McpServerConfig> {
        Ok(McpServerConfig {
            name: name.to_string(),
            command: resolve_path(base_dir, &self.command),
            args: self.args.clone(),
            cwd: self.cwd.as_ref().map(|path| resolve_path(base_dir, path)),
            env: self.env.clone(),
        })
    }
}

/// Loads MCP server definitions from a JSON file.
pub fn load_mcp_config(path: impl AsRef<Path>) -> NestResult<McpConfigFile> {
    let path = path.as_ref();
    let raw = std::fs::read_to_string(path).map_err(|error| {
        mcp_to_nest(format!(
            "failed to read MCP config {}: {error}",
            path.display()
        ))
    })?;
    let config: McpConfigFile = serde_json::from_str(&raw).map_err(|error| {
        mcp_to_nest(format!(
            "failed to parse MCP config {}: {error}",
            path.display()
        ))
    })?;
    Ok(config)
}

fn resolve_path(base_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base_dir.join(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_cursor_mcp_json() {
        let json = r#"{
            "mcpServers": {
                "nest-memory": {
                    "command": ".venv/bin/python",
                    "args": ["tools/mcp_memory_server.py"],
                    "cwd": "/data/projects/nest"
                }
            }
        }"#;
        let config: McpConfigFile = serde_json::from_str(json).unwrap();
        let entry = config.mcp_servers.get("nest-memory").unwrap();
        let server = entry
            .to_config("nest-memory", Path::new("/data/projects/nest"))
            .unwrap();
        assert_eq!(server.name, "nest-memory");
        assert_eq!(
            server.command,
            PathBuf::from("/data/projects/nest/.venv/bin/python")
        );
    }
}
