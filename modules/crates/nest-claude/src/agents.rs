//! Request/response types for the Agents API (`/v1/agents`), part of
//! Managed Agents.
//!
//! Every Agents endpoint requires the `anthropic-beta: managed-agents-2026-04-01`
//! header — [`ClaudeClient`](crate::ClaudeClient) sends it per-request, since
//! it doesn't apply to `/v1/messages` or the Skills endpoints.
//!
//! v1 covers only the agent resource itself (create/list/get/update/archive).
//! Sessions, environments, vaults, and the rest of the Managed Agents surface
//! are not yet covered.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::util::percent_encode;

/// Speed tier for an agent's model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Speed {
    /// Default inference speed.
    Standard,
    /// Higher tokens/sec at premium pricing (Opus 4.8/4.7 only).
    Fast,
}

/// The model to run an agent on: a bare model ID, or an ID with a speed override.
///
/// Requests accept either shape; responses always echo the object form (see
/// [`AgentModelInfo`]).
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum AgentModel {
    /// A bare model ID, e.g. `"claude-opus-4-8"`.
    Id(String),
    /// A model ID with an explicit speed tier.
    WithSpeed {
        /// The model ID.
        id: String,
        /// The requested speed tier.
        speed: Speed,
    },
}

impl AgentModel {
    /// A bare model ID.
    pub fn id(id: impl Into<String>) -> Self {
        Self::Id(id.into())
    }

    /// A model ID with an explicit speed tier.
    pub fn with_speed(id: impl Into<String>, speed: Speed) -> Self {
        Self::WithSpeed {
            id: id.into(),
            speed,
        }
    }
}

impl From<&str> for AgentModel {
    fn from(value: &str) -> Self {
        Self::Id(value.to_string())
    }
}

impl From<String> for AgentModel {
    fn from(value: String) -> Self {
        Self::Id(value)
    }
}

/// The resolved model info on an [`Agent`] response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentModelInfo {
    /// The model ID.
    pub id: String,
    /// The resolved speed tier.
    pub speed: Option<Speed>,
}

/// Whether a tool executes automatically or waits for `user.tool_confirmation`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PermissionPolicy {
    /// The tool executes automatically (default).
    #[serde(rename = "always_allow")]
    AlwaysAllow,
    /// The session pauses for a `user.tool_confirmation` event before running.
    #[serde(rename = "always_ask")]
    AlwaysAsk,
}

/// Default enablement/permission applied to every tool in a toolset.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolConfig {
    /// Whether the tool is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// The permission policy for this tool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_policy: Option<PermissionPolicy>,
}

/// A per-tool override within a toolset's `configs`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamedToolConfig {
    /// The tool name being overridden (e.g. `"bash"`).
    pub name: String,
    /// Whether the tool is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// The permission policy for this tool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_policy: Option<PermissionPolicy>,
}

/// A tool made available to an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AgentTool {
    /// The prebuilt Claude Agent toolset (bash, read, write, edit, glob, grep,
    /// web_fetch, web_search).
    #[serde(rename = "agent_toolset_20260401")]
    AgentToolset {
        /// Default enablement/permission for every tool in the set.
        #[serde(skip_serializing_if = "Option::is_none")]
        default_config: Option<ToolConfig>,
        /// Per-tool overrides.
        #[serde(skip_serializing_if = "Option::is_none")]
        configs: Option<Vec<NamedToolConfig>>,
    },
    /// Tools exposed by a connected MCP server (see [`McpServerDefinition`]).
    #[serde(rename = "mcp_toolset")]
    McpToolset {
        /// The MCP server's `name`, as declared in the agent's `mcp_servers`.
        mcp_server_name: String,
        /// Default enablement/permission for every tool the server exposes.
        #[serde(skip_serializing_if = "Option::is_none")]
        default_config: Option<ToolConfig>,
        /// Per-tool overrides.
        #[serde(skip_serializing_if = "Option::is_none")]
        configs: Option<Vec<NamedToolConfig>>,
    },
    /// A client-executed custom tool.
    #[serde(rename = "custom")]
    Custom {
        /// The tool's name.
        name: String,
        /// A description of when and how to use the tool.
        description: String,
        /// JSON Schema for the tool's input.
        input_schema: serde_json::Value,
    },
}

impl AgentTool {
    /// The prebuilt Claude Agent toolset with default settings.
    pub fn agent_toolset() -> Self {
        Self::AgentToolset {
            default_config: None,
            configs: None,
        }
    }

    /// Tools exposed by the named MCP server.
    pub fn mcp_toolset(mcp_server_name: impl Into<String>) -> Self {
        Self::McpToolset {
            mcp_server_name: mcp_server_name.into(),
            default_config: None,
            configs: None,
        }
    }

    /// A client-executed custom tool.
    pub fn custom(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: serde_json::Value,
    ) -> Self {
        Self::Custom {
            name: name.into(),
            description: description.into(),
            input_schema,
        }
    }
}

/// A skill reference attached to an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AgentSkillRef {
    /// A prebuilt Anthropic skill (e.g. `"xlsx"`, `"docx"`, `"pptx"`, `"pdf"`).
    #[serde(rename = "anthropic")]
    Anthropic {
        /// The skill's name.
        skill_id: String,
        /// A specific version, or omit for the latest.
        #[serde(skip_serializing_if = "Option::is_none")]
        version: Option<String>,
    },
    /// A custom skill created via the Skills API.
    #[serde(rename = "custom")]
    Custom {
        /// The skill's id (e.g. `"skill_..."`).
        skill_id: String,
        /// A specific version, or omit for the latest.
        #[serde(skip_serializing_if = "Option::is_none")]
        version: Option<String>,
    },
}

impl AgentSkillRef {
    /// A prebuilt Anthropic skill.
    pub fn anthropic(skill_id: impl Into<String>) -> Self {
        Self::Anthropic {
            skill_id: skill_id.into(),
            version: None,
        }
    }

    /// A custom skill created via the Skills API.
    pub fn custom(skill_id: impl Into<String>) -> Self {
        Self::Custom {
            skill_id: skill_id.into(),
            version: None,
        }
    }
}

/// A connected MCP server declaration (credentials are supplied separately,
/// via a vault, at session time — not modeled here).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerDefinition {
    /// A unique name for this server, referenced by [`AgentTool::McpToolset`].
    pub name: String,
    #[serde(rename = "type")]
    kind: McpServerKind,
    /// The MCP server's endpoint URL (Streamable HTTP transport).
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum McpServerKind {
    Url,
}

impl McpServerDefinition {
    /// Declares a URL-based MCP server.
    pub fn url(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: McpServerKind::Url,
            url: url.into(),
        }
    }
}

/// A member of a coordinator's sub-agent roster.
///
/// Serializes/deserializes by hand rather than via `#[serde(untagged)]`,
/// since the wire format mixes a bare string variant with two differently
/// tagged object variants (`{"type":"agent",...}` / `{"type":"self"}`) —
/// shapes `#[serde(tag = "type")]` (needs every variant to carry the tag,
/// including the bare-string one) and plain `#[serde(untagged)]` (emits
/// `null` for a fieldless variant like `SelfRef`, not `{"type":"self"}`)
/// can't both express at once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentRef {
    /// The latest version of the named agent.
    Id(String),
    /// A specific (or latest, if `version` is omitted) version of another agent.
    Versioned {
        /// The referenced agent's id.
        id: String,
        /// A specific version, or omit for the latest.
        version: Option<u64>,
    },
    /// The coordinator delegating to a copy of itself.
    SelfRef,
}

impl Serialize for AgentRef {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;

        match self {
            Self::Id(id) => serializer.serialize_str(id),
            Self::Versioned { id, version } => {
                let mut map =
                    serializer.serialize_map(Some(if version.is_some() { 3 } else { 2 }))?;
                map.serialize_entry("type", "agent")?;
                map.serialize_entry("id", id)?;
                if let Some(version) = version {
                    map.serialize_entry("version", version)?;
                }
                map.end()
            }
            Self::SelfRef => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("type", "self")?;
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for AgentRef {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error as _;

        match serde_json::Value::deserialize(deserializer)? {
            serde_json::Value::String(id) => Ok(Self::Id(id)),
            serde_json::Value::Object(map) => {
                match map.get("type").and_then(serde_json::Value::as_str) {
                    Some("self") => Ok(Self::SelfRef),
                    Some("agent") => {
                        let id = map
                            .get("id")
                            .and_then(serde_json::Value::as_str)
                            .map(str::to_string)
                            .ok_or_else(|| D::Error::missing_field("id"))?;
                        let version = map.get("version").and_then(serde_json::Value::as_u64);
                        Ok(Self::Versioned { id, version })
                    }
                    Some(other) => Err(D::Error::unknown_variant(other, &["agent", "self"])),
                    None => Err(D::Error::missing_field("type")),
                }
            }
            _ => Err(D::Error::custom(
                "expected a string or an object for AgentRef",
            )),
        }
    }
}

/// Multi-agent coordinator configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Multiagent {
    #[serde(rename = "type")]
    kind: MultiagentKind,
    /// The roster of agents this coordinator may delegate to (1–20 entries).
    pub agents: Vec<AgentRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum MultiagentKind {
    Coordinator,
}

impl Multiagent {
    /// Creates a coordinator with the given roster.
    pub fn coordinator(agents: Vec<AgentRef>) -> Self {
        Self {
            kind: MultiagentKind::Coordinator,
            agents,
        }
    }
}

/// A persisted, versioned Managed Agents agent configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    /// Unique identifier for the agent.
    pub id: String,
    /// The version number; starts at 1 and increments on each update.
    pub version: u64,
    /// Human-readable name.
    pub name: String,
    /// Description of the agent's purpose.
    pub description: Option<String>,
    /// The system prompt.
    pub system: Option<String>,
    /// RFC 3339 creation timestamp.
    pub created_at: String,
    /// RFC 3339 last-update timestamp.
    pub updated_at: String,
    /// RFC 3339 archival timestamp, if archived.
    pub archived_at: Option<String>,
    /// The resolved model.
    pub model: AgentModelInfo,
    /// Tools available to the agent.
    #[serde(default)]
    pub tools: Vec<AgentTool>,
    /// Skills available to the agent.
    #[serde(default)]
    pub skills: Vec<AgentSkillRef>,
    /// Connected MCP servers.
    #[serde(default)]
    pub mcp_servers: Vec<McpServerDefinition>,
    /// Arbitrary key-value metadata.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    /// Coordinator roster, if this agent delegates to sub-agents.
    pub multiagent: Option<Multiagent>,
}

/// Builder for `POST /v1/agents`.
#[derive(Debug, Clone, Serialize)]
pub struct CreateAgentRequest {
    name: String,
    model: AgentModel,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<AgentTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    skills: Option<Vec<AgentSkillRef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mcp_servers: Option<Vec<McpServerDefinition>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    multiagent: Option<Multiagent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<HashMap<String, String>>,
}

impl CreateAgentRequest {
    /// Creates a request builder with the required `name` and `model`.
    pub fn new(name: impl Into<String>, model: impl Into<AgentModel>) -> Self {
        Self {
            name: name.into(),
            model: model.into(),
            description: None,
            system: None,
            tools: None,
            skills: None,
            mcp_servers: None,
            multiagent: None,
            metadata: None,
        }
    }

    /// Sets the agent's description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the system prompt.
    pub fn system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(system.into());
        self
    }

    /// Sets the available tools.
    pub fn tools(mut self, tools: Vec<AgentTool>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Sets the available skills.
    pub fn skills(mut self, skills: Vec<AgentSkillRef>) -> Self {
        self.skills = Some(skills);
        self
    }

    /// Sets connected MCP servers.
    pub fn mcp_servers(mut self, mcp_servers: Vec<McpServerDefinition>) -> Self {
        self.mcp_servers = Some(mcp_servers);
        self
    }

    /// Sets the coordinator roster for a multi-agent setup.
    pub fn multiagent(mut self, multiagent: Multiagent) -> Self {
        self.multiagent = Some(multiagent);
        self
    }

    /// Sets metadata (max 16 pairs, keys ≤64 chars, values ≤512 chars).
    pub fn metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Builder for `POST /v1/agents/{agent_id}` (update).
///
/// Fields left unset are preserved. For `description`/`system`, pass an empty
/// string via `.description("")`/`.system("")` (or the `clear_*` helpers) to
/// clear them; for `tools`/`skills`/`mcp_servers`, pass an empty `Vec`.
/// `metadata` is a patch: set a key to `Some(value)` to upsert it, or `None`
/// to delete it.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateAgentRequest {
    version: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    model: Option<AgentModel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<AgentTool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    skills: Option<Vec<AgentSkillRef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mcp_servers: Option<Vec<McpServerDefinition>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    multiagent: Option<Multiagent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<HashMap<String, Option<String>>>,
}

impl UpdateAgentRequest {
    /// Creates a request builder pinned to the agent's current `version`
    /// (optimistic concurrency lock — fetch it via `get_agent` first).
    pub fn new(version: u64) -> Self {
        Self {
            version,
            name: None,
            description: None,
            model: None,
            system: None,
            tools: None,
            skills: None,
            mcp_servers: None,
            multiagent: None,
            metadata: None,
        }
    }

    /// Sets a new (non-empty) name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Sets the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Clears the description.
    pub fn clear_description(mut self) -> Self {
        self.description = Some(String::new());
        self
    }

    /// Sets the model.
    pub fn model(mut self, model: impl Into<AgentModel>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Sets the system prompt.
    pub fn system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(system.into());
        self
    }

    /// Clears the system prompt.
    pub fn clear_system(mut self) -> Self {
        self.system = Some(String::new());
        self
    }

    /// Replaces the available tools.
    pub fn tools(mut self, tools: Vec<AgentTool>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Clears all tools.
    pub fn clear_tools(mut self) -> Self {
        self.tools = Some(Vec::new());
        self
    }

    /// Replaces the available skills.
    pub fn skills(mut self, skills: Vec<AgentSkillRef>) -> Self {
        self.skills = Some(skills);
        self
    }

    /// Clears all skills.
    pub fn clear_skills(mut self) -> Self {
        self.skills = Some(Vec::new());
        self
    }

    /// Replaces the connected MCP servers.
    pub fn mcp_servers(mut self, mcp_servers: Vec<McpServerDefinition>) -> Self {
        self.mcp_servers = Some(mcp_servers);
        self
    }

    /// Clears all connected MCP servers.
    pub fn clear_mcp_servers(mut self) -> Self {
        self.mcp_servers = Some(Vec::new());
        self
    }

    /// Sets the coordinator roster for a multi-agent setup.
    pub fn multiagent(mut self, multiagent: Multiagent) -> Self {
        self.multiagent = Some(multiagent);
        self
    }

    /// Patches metadata: `Some(value)` upserts a key, `None` deletes it.
    pub fn metadata(mut self, metadata: HashMap<String, Option<String>>) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Query parameters for `GET /v1/agents`.
#[derive(Debug, Clone, Default)]
pub struct ListAgentsParams {
    limit: Option<u32>,
    page: Option<String>,
    include_archived: Option<bool>,
    created_at_gte: Option<String>,
    created_at_lte: Option<String>,
}

impl ListAgentsParams {
    /// Creates empty (default) list parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the page size (max 100, defaults to 20 server-side).
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the pagination token from a previous response's `next_page`.
    pub fn page(mut self, page: impl Into<String>) -> Self {
        self.page = Some(page.into());
        self
    }

    /// Includes archived agents in the results (defaults to `false` server-side).
    pub fn include_archived(mut self, include_archived: bool) -> Self {
        self.include_archived = Some(include_archived);
        self
    }

    /// Filters to agents created at or after this RFC 3339 timestamp.
    pub fn created_at_gte(mut self, timestamp: impl Into<String>) -> Self {
        self.created_at_gte = Some(timestamp.into());
        self
    }

    /// Filters to agents created at or before this RFC 3339 timestamp.
    pub fn created_at_lte(mut self, timestamp: impl Into<String>) -> Self {
        self.created_at_lte = Some(timestamp.into());
        self
    }

    pub(crate) fn to_query_string(&self) -> String {
        let mut params = Vec::new();
        if let Some(limit) = self.limit {
            params.push(format!("limit={limit}"));
        }
        if let Some(page) = &self.page {
            params.push(format!("page={}", percent_encode(page)));
        }
        if let Some(include_archived) = self.include_archived {
            params.push(format!("include_archived={include_archived}"));
        }
        if let Some(timestamp) = &self.created_at_gte {
            params.push(format!(
                "created_at{}={}",
                percent_encode("[gte]"),
                percent_encode(timestamp)
            ));
        }
        if let Some(timestamp) = &self.created_at_lte {
            params.push(format!(
                "created_at{}={}",
                percent_encode("[lte]"),
                percent_encode(timestamp)
            ));
        }
        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
    }
}

/// One page of `GET /v1/agents`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentListPage {
    /// The agents in this page.
    pub data: Vec<Agent>,
    /// Pagination token for the next page, when present.
    pub next_page: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_id_serializes_as_bare_string() {
        assert_eq!(
            serde_json::to_value(AgentModel::id("claude-opus-4-8")).unwrap(),
            "claude-opus-4-8"
        );
    }

    #[test]
    fn model_with_speed_serializes_as_object() {
        let json =
            serde_json::to_value(AgentModel::with_speed("claude-opus-4-8", Speed::Fast)).unwrap();
        assert_eq!(json["id"], "claude-opus-4-8");
        assert_eq!(json["speed"], "fast");
    }

    #[test]
    fn create_agent_request_omits_unset_optional_fields() {
        let request = CreateAgentRequest::new("My Agent", "claude-opus-4-8");
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["name"], "My Agent");
        assert_eq!(json["model"], "claude-opus-4-8");
        assert!(json.get("system").is_none());
        assert!(json.get("tools").is_none());
        assert!(json.get("metadata").is_none());
    }

    #[test]
    fn agent_tool_variants_use_documented_tags() {
        let toolset = serde_json::to_value(AgentTool::agent_toolset()).unwrap();
        assert_eq!(toolset["type"], "agent_toolset_20260401");

        let mcp = serde_json::to_value(AgentTool::mcp_toolset("github")).unwrap();
        assert_eq!(mcp["type"], "mcp_toolset");
        assert_eq!(mcp["mcp_server_name"], "github");

        let custom = serde_json::to_value(AgentTool::custom(
            "run_tests",
            "Run the test suite",
            serde_json::json!({"type": "object"}),
        ))
        .unwrap();
        assert_eq!(custom["type"], "custom");
    }

    #[test]
    fn update_agent_request_clear_helpers_send_empty_values() {
        let request = UpdateAgentRequest::new(1).clear_system().clear_tools();
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["version"], 1);
        assert_eq!(json["system"], "");
        assert_eq!(json["tools"], serde_json::json!([]));
        assert!(json.get("name").is_none());
    }

    #[test]
    fn update_agent_metadata_patch_supports_delete() {
        let mut metadata = HashMap::new();
        metadata.insert("foo".to_string(), Some("bar".to_string()));
        metadata.insert("old".to_string(), None);
        let request = UpdateAgentRequest::new(2).metadata(metadata);
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["metadata"]["foo"], "bar");
        assert!(json["metadata"]["old"].is_null());
    }

    #[test]
    fn agent_ref_variants_round_trip() {
        assert_eq!(
            serde_json::to_value(AgentRef::Id("agent_abc".into())).unwrap(),
            "agent_abc"
        );
        assert_eq!(
            serde_json::to_value(AgentRef::SelfRef).unwrap(),
            serde_json::json!({"type": "self"})
        );
        let versioned = serde_json::to_value(AgentRef::Versioned {
            id: "agent_def".into(),
            version: Some(4),
        })
        .unwrap();
        assert_eq!(versioned["id"], "agent_def");
        assert_eq!(versioned["version"], 4);
    }

    #[test]
    fn list_agents_query_string_includes_bracket_params() {
        let query = ListAgentsParams::new()
            .limit(10)
            .include_archived(true)
            .created_at_gte("2026-01-01T00:00:00Z")
            .to_query_string();
        assert!(query.contains("limit=10"));
        assert!(query.contains("include_archived=true"));
        assert!(query.contains("created_at%5Bgte%5D=2026-01-01T00%3A00%3A00Z"));
    }

    #[test]
    fn agent_response_deserializes_with_full_object() {
        let agent: Agent = serde_json::from_value(serde_json::json!({
            "id": "agent_011CZkYpogX7uDKUyvBTophP",
            "type": "agent",
            "version": 1,
            "name": "My First Agent",
            "description": "A general-purpose starter agent.",
            "created_at": "2026-03-15T10:00:00Z",
            "updated_at": "2026-03-15T10:00:00Z",
            "archived_at": null,
            "model": {"id": "claude-sonnet-4-6", "speed": "standard"},
            "system": "You are a helpful agent.",
            "tools": [{"type": "agent_toolset_20260401"}],
            "skills": [{"type": "anthropic", "skill_id": "xlsx", "version": "1"}],
            "mcp_servers": [{"name": "example-mcp", "type": "url", "url": "https://example.com/sse"}],
            "metadata": {"foo": "bar"}
        }))
        .unwrap();

        assert_eq!(agent.id, "agent_011CZkYpogX7uDKUyvBTophP");
        assert_eq!(agent.version, 1);
        assert_eq!(agent.model.id, "claude-sonnet-4-6");
        assert_eq!(agent.tools.len(), 1);
        assert_eq!(agent.metadata.get("foo"), Some(&"bar".to_string()));
        assert!(agent.archived_at.is_none());
        assert!(agent.multiagent.is_none());
    }

    #[test]
    fn agent_round_trips_through_serialize_and_deserialize() {
        let agent: Agent = serde_json::from_value(serde_json::json!({
            "id": "agent_011CZkYpogX7uDKUyvBTophP",
            "type": "agent",
            "version": 1,
            "name": "My First Agent",
            "description": null,
            "created_at": "2026-03-15T10:00:00Z",
            "updated_at": "2026-03-15T10:00:00Z",
            "archived_at": null,
            "model": {"id": "claude-sonnet-4-6", "speed": "standard"},
            "system": null,
            "tools": [],
            "skills": [{"type": "custom", "skill_id": "skill_abc", "version": null}],
            "mcp_servers": [],
            "metadata": {}
        }))
        .unwrap();

        let json = serde_json::to_value(&agent).unwrap();
        let reparsed: Agent = serde_json::from_value(json).unwrap();
        assert_eq!(reparsed.id, agent.id);
        assert_eq!(reparsed.model.id, agent.model.id);
        assert_eq!(reparsed.skills.len(), 1);
    }
}
