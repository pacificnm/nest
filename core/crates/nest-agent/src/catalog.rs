//! Tool catalog probing for UI hosts.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use nest_file::{FileService, FileServiceConfig};

use crate::composite::CompositeToolSource;
use crate::tool::AgentTool;
use crate::tools::{SharedMcpHub, ToolSource};
use crate::NestResult;

/// Lists all agent tools from MCP servers and optional native file tools.
pub async fn probe_tools(
    mcp_config: &Path,
    mcp_servers: &[String],
    project_root: Option<PathBuf>,
    extra_env: HashMap<String, String>,
) -> NestResult<Vec<AgentTool>> {
    let mcp_servers: Vec<String> = mcp_servers
        .iter()
        .filter(|name| name.as_str() != "nest-file")
        .cloned()
        .collect();

    let hub = SharedMcpHub::from_config_file_with_env(mcp_config, Some(&mcp_servers), extra_env)
        .await?;

    let mut source = CompositeToolSource::new(hub);
    if let Some(root) = project_root {
        if let Ok(files) = FileService::with_config(FileServiceConfig::scoped(root)) {
            source = source.with_files(files);
        }
    }

    source.list_tools().await
}
