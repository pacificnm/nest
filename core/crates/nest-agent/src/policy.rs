//! Read-only tool auto-run policy.

use crate::config::AutoRunPolicy;
use nest_mcp::McpTool;

/// Returns true when the tool may run without user approval.
pub fn is_read_only_tool(server: &str, tool: &str) -> bool {
    matches!(
        (server, tool),
        ("nest-memory", "search_project_memory")
            | ("nest-knowledge", "search_knowledge_base")
            | ("nest-knowledge", "list_knowledge_collections")
            | ("nest-context-memory", "search_context_memory")
            | ("nest-context-memory", "list_context_memory")
            | ("nest-context-memory", "get_context_memory")
    )
}

/// Returns true when the given MCP tool may run under the policy.
pub fn may_auto_run(policy: AutoRunPolicy, tool: &McpTool) -> bool {
    match policy {
        AutoRunPolicy::ReadOnlyOnly => is_read_only_tool(&tool.server, &tool.name),
        AutoRunPolicy::Ask => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nest_mcp::McpTool;
    use serde_json::json;

    fn sample_tool(server: &str, name: &str) -> McpTool {
        McpTool {
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
        assert!(may_auto_run(AutoRunPolicy::ReadOnlyOnly, &tool));
    }

    #[test]
    fn write_context_save_blocked() {
        let tool = sample_tool("nest-context-memory", "save_context_memory");
        assert!(!may_auto_run(AutoRunPolicy::ReadOnlyOnly, &tool));
    }
}
