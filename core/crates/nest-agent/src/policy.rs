//! Read-only tool auto-run policy.

use crate::config::AgentConfig;
use crate::file_tools::FILE_SERVER;
use crate::tool::AgentTool;

/// Returns true when the tool belongs to the Nest file MCP server.
pub fn is_file_tool(server: &str, _tool: &str) -> bool {
    server == FILE_SERVER
}

/// Returns true when the tool is a read-only Nest memory search tool.
pub fn is_read_only_tool(server: &str, tool: &str) -> bool {
    matches!(
        (server, tool),
        ("nest-memory", "search_project_memory")
            | ("nest-knowledge", "search_knowledge_base")
            | ("nest-knowledge", "list_knowledge_collections")
            | ("nest-context-memory", "search_context_memory")
            | ("nest-context-memory", "list_context_memory")
            | ("nest-context-memory", "get_context_memory")
            | (FILE_SERVER, "read_file")
            | (FILE_SERVER, "list_directory")
            | (FILE_SERVER, "search_files")
    )
}

/// Returns true when the tool is `save_context_memory` and the config allows it.
pub fn is_save_context_tool(server: &str, tool: &str) -> bool {
    server == "nest-context-memory" && tool == "save_context_memory"
}

/// Returns true when the tool mutates files and the config allows it.
pub fn is_file_write_tool(server: &str, tool: &str) -> bool {
    server == FILE_SERVER
        && matches!(
            tool,
            "write_file" | "update_file" | "delete_path" | "create_directory"
        )
}

/// Returns true when the given tool may run without user approval.
pub fn may_auto_run(config: &AgentConfig, tool: &AgentTool) -> bool {
    match config.auto_run_policy {
        crate::config::AutoRunPolicy::ReadOnlyOnly => {
            is_read_only_tool(&tool.server, &tool.name)
                || (config.allow_save_context && is_save_context_tool(&tool.server, &tool.name))
                || (config.allow_file_writes && is_file_write_tool(&tool.server, &tool.name))
        }
        crate::config::AutoRunPolicy::Ask => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AgentConfig;
    use crate::tool::{AgentTool, ToolOrigin};
    use serde_json::json;

    fn sample_tool(server: &str, name: &str) -> AgentTool {
        AgentTool {
            origin: ToolOrigin::Mcp,
            server: server.into(),
            name: name.into(),
            qualified_name: format!("{server}/{name}"),
            description: String::new(),
            input_schema: json!({}),
        }
    }

    #[test]
    fn read_only_memory_search_allowed() {
        let tool = sample_tool("nest-memory", "search_project_memory");
        assert!(may_auto_run(&AgentConfig::default(), &tool));
    }

    #[test]
    fn write_context_save_blocked_by_default() {
        let tool = sample_tool("nest-context-memory", "save_context_memory");
        assert!(!may_auto_run(&AgentConfig::default(), &tool));
    }

    #[test]
    fn write_context_save_allowed_when_configured() {
        let tool = sample_tool("nest-context-memory", "save_context_memory");
        let config = AgentConfig::default().with_allow_save_context(true);
        assert!(may_auto_run(&config, &tool));
    }

    #[test]
    fn file_read_allowed_by_default() {
        let tool = sample_tool("nest-file", "read_file");
        assert!(may_auto_run(&AgentConfig::default(), &tool));
    }

    #[test]
    fn file_search_allowed_by_default() {
        let tool = sample_tool("nest-file", "search_files");
        assert!(may_auto_run(&AgentConfig::default(), &tool));
    }

    #[test]
    fn file_write_blocked_by_default() {
        let tool = sample_tool("nest-file", "write_file");
        assert!(!may_auto_run(&AgentConfig::default(), &tool));
    }

    #[test]
    fn file_write_allowed_when_configured() {
        let tool = sample_tool("nest-file", "write_file");
        let config = AgentConfig::default().with_allow_file_writes(true);
        assert!(may_auto_run(&config, &tool));
    }
}
