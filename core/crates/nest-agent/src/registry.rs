//! Agent tool registry and model name mapping.

use std::collections::HashMap;

use nest_ai::ToolDefinition;

use crate::tool::AgentTool;

/// Maps agent tools to model-visible function names.
#[derive(Debug, Clone)]
pub struct ToolRegistry {
    tools: Vec<AgentTool>,
    model_to_qualified: HashMap<String, String>,
    model_to_tool: HashMap<String, AgentTool>,
}

impl ToolRegistry {
    /// Builds a registry from agent tool metadata.
    pub fn from_tools(tools: Vec<AgentTool>) -> Self {
        let mut model_to_qualified = HashMap::new();
        let mut model_to_tool = HashMap::new();
        for tool in &tools {
            let model_name = model_tool_name(&tool.server, &tool.name);
            model_to_qualified.insert(model_name.clone(), tool.qualified_name.clone());
            model_to_tool.insert(model_name, tool.clone());
        }
        Self {
            tools,
            model_to_qualified,
            model_to_tool,
        }
    }

    /// Tools in registry order.
    pub fn tools(&self) -> &[AgentTool] {
        &self.tools
    }

    /// Tool definitions for the model request.
    pub fn definitions(&self) -> Vec<ToolDefinition> {
        self.tools
            .iter()
            .map(|tool| {
                ToolDefinition::new(
                    model_tool_name(&tool.server, &tool.name),
                    &tool.description,
                    tool.input_schema.clone(),
                )
            })
            .collect()
    }

    /// Resolves a model function name to the qualified name (`server/tool`).
    pub fn qualified_name(&self, model_name: &str) -> Option<&str> {
        if let Some(qualified) = self.model_to_qualified.get(model_name) {
            return Some(qualified.as_str());
        }
        self.tools
            .iter()
            .find(|tool| tool.name == model_name)
            .map(|tool| tool.qualified_name.as_str())
    }

    /// Returns tool metadata for a model function name.
    pub fn agent_tool(&self, model_name: &str) -> Option<&AgentTool> {
        if let Some(tool) = self.model_to_tool.get(model_name) {
            return Some(tool);
        }
        self.tools.iter().find(|tool| tool.name == model_name)
    }
}

/// Builds the stable model-visible tool name (`nest_memory__search_project_memory`).
pub fn model_tool_name(server: &str, tool: &str) -> String {
    format!("{}__{}", server.replace('-', "_"), tool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tool::ToolOrigin;
    use serde_json::json;

    #[test]
    fn model_name_sanitizes_server_dashes() {
        assert_eq!(
            model_tool_name("nest-memory", "search_project_memory"),
            "nest_memory__search_project_memory"
        );
    }

    #[test]
    fn definitions_use_model_names() {
        let registry = ToolRegistry::from_tools(vec![AgentTool {
            origin: ToolOrigin::Mcp,
            server: "nest-memory".into(),
            name: "search_project_memory".into(),
            qualified_name: "nest-memory/search_project_memory".into(),
            description: "Search docs".into(),
            input_schema: json!({"type": "object"}),
        }]);
        assert_eq!(registry.definitions()[0].name, "nest_memory__search_project_memory");
        assert_eq!(
            registry.qualified_name("nest_memory__search_project_memory"),
            Some("nest-memory/search_project_memory")
        );
    }

    #[test]
    fn resolves_bare_tool_name() {
        let registry = ToolRegistry::from_tools(vec![AgentTool {
            origin: ToolOrigin::Mcp,
            server: "nest-knowledge".into(),
            name: "list_knowledge_collections".into(),
            qualified_name: "nest-knowledge/list_knowledge_collections".into(),
            description: "List".into(),
            input_schema: json!({"type": "object"}),
        }]);
        assert_eq!(
            registry.qualified_name("list_knowledge_collections"),
            Some("nest-knowledge/list_knowledge_collections")
        );
    }
}
