//! Request/response types for the Sessions API (`/v1/sessions`), part of
//! Managed Agents.
//!
//! Every Sessions endpoint requires the `anthropic-beta: managed-agents-2026-04-01`
//! header — [`ClaudeClient`](crate::ClaudeClient) sends it per-request.
//!
//! v1 covers only the session resource's lifecycle (create/list/get/update/
//! delete/archive). Session Events (sending messages, streaming replies) are
//! not yet covered — a created session is inspectable but not yet usable for
//! a live conversation from this crate.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::util::percent_encode;

/// A session's current lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    /// Awaiting input — either a new `user.message`, or a queued
    /// `user.custom_tool_result`/`user.tool_confirmation`.
    Idle,
    /// Actively doing work.
    Running,
    /// Recovering from a retryable error; will resume automatically.
    Rescheduling,
    /// Terminated — an irreversible, unusable end state.
    Terminated,
}

/// Which agent (and version) a session runs, on `POST /v1/sessions`.
///
/// Serializes by hand rather than via `#[serde(untagged)]`, since the wire
/// format mixes a bare string variant (latest version) with a tagged object
/// variant (`{"type":"agent",...}`) — the same shape `agents::AgentRef`
/// handles, minus the `self` variant, which only applies inside a
/// multi-agent coordinator's roster, not a session.
///
/// Request-only: there is no corresponding `Deserialize` impl, since
/// [`Session::agent`] echoes back the *resolved* agent configuration, not a
/// reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionAgentRef {
    /// The latest version of the named agent.
    Id(String),
    /// A specific (or latest, if `version` is omitted) version of the agent.
    Versioned {
        /// The referenced agent's id.
        id: String,
        /// A specific version, or omit for the latest.
        version: Option<u64>,
    },
}

impl SessionAgentRef {
    /// The latest version of the named agent.
    pub fn id(id: impl Into<String>) -> Self {
        Self::Id(id.into())
    }

    /// A specific version of the named agent.
    pub fn versioned(id: impl Into<String>, version: u64) -> Self {
        Self::Versioned {
            id: id.into(),
            version: Some(version),
        }
    }
}

impl From<&str> for SessionAgentRef {
    fn from(value: &str) -> Self {
        Self::Id(value.to_string())
    }
}

impl From<String> for SessionAgentRef {
    fn from(value: String) -> Self {
        Self::Id(value)
    }
}

impl Serialize for SessionAgentRef {
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
        }
    }
}

/// A persisted, running (or terminated) Managed Agents session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Unique identifier for the session.
    pub id: String,
    /// Human-readable title, if set.
    pub title: Option<String>,
    /// The session's current lifecycle state.
    pub status: SessionStatus,
    /// The environment this session runs in.
    pub environment_id: String,
    /// RFC 3339 creation timestamp.
    pub created_at: String,
    /// RFC 3339 last-update timestamp.
    pub updated_at: String,
    /// RFC 3339 archival timestamp, if archived.
    pub archived_at: Option<String>,
    /// The resolved agent configuration this session runs (not modeled in
    /// detail — round-tripped as-is).
    #[serde(default)]
    pub agent: serde_json::Value,
    /// Attached files, repos, and memory stores (not modeled in detail).
    #[serde(default)]
    pub resources: serde_json::Value,
    /// User-provided key-value pairs.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    /// Token usage statistics (not modeled in detail).
    #[serde(default)]
    pub usage: serde_json::Value,
}

/// Builder for `POST /v1/sessions`.
#[derive(Debug, Clone, Serialize)]
pub struct CreateSessionRequest {
    agent: SessionAgentRef,
    environment_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
}

impl CreateSessionRequest {
    /// Creates a request builder with the required `agent` and `environment_id`.
    pub fn new(agent: impl Into<SessionAgentRef>, environment_id: impl Into<String>) -> Self {
        Self {
            agent: agent.into(),
            environment_id: environment_id.into(),
            title: None,
        }
    }

    /// Sets a human-readable title (appears in logs/dashboards).
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
}

/// Builder for `POST /v1/sessions/{session_id}` (update).
///
/// v1 supports `title` and `metadata` only. Updating a session's
/// `agent.tools`/`agent.mcp_servers`/`vault_ids` mid-session (valid only
/// while the session is `idle`) is not yet covered.
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateSessionRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<HashMap<String, String>>,
}

impl UpdateSessionRequest {
    /// Creates an empty request builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a new title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Replaces the session's metadata (max 8 pairs).
    pub fn metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Response from `DELETE /v1/sessions/{session_id}`.
#[derive(Debug, Clone, Deserialize)]
pub struct SessionDeleted {
    /// The id of the deleted session.
    pub id: String,
}

/// Sort order for `GET /v1/sessions`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionOrder {
    /// Oldest first.
    Asc,
    /// Newest first.
    Desc,
}

impl SessionOrder {
    fn as_str(self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

/// Query parameters for `GET /v1/sessions`.
#[derive(Debug, Clone, Default)]
pub struct ListSessionsParams {
    limit: Option<u32>,
    page: Option<String>,
    order: Option<SessionOrder>,
}

impl ListSessionsParams {
    /// Creates empty (default) list parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the page size (max 100, defaults to 20 server-side).
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the pagination token from a previous response's `next_page` (or
    /// `prev_page`, to page backward).
    pub fn page(mut self, page: impl Into<String>) -> Self {
        self.page = Some(page.into());
        self
    }

    /// Sets the sort order. A cursor encodes the order of the request that
    /// produced it — reusing it with a different order returns `400`.
    pub fn order(mut self, order: SessionOrder) -> Self {
        self.order = Some(order);
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
        if let Some(order) = self.order {
            params.push(format!("order={}", order.as_str()));
        }
        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
    }
}

/// One page of `GET /v1/sessions`.
///
/// Sessions is the one Managed Agents list endpoint that supports backward
/// pagination — pass [`SessionListPage::prev_page`] back into
/// [`ListSessionsParams::page`] to go back a page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionListPage {
    /// The sessions in this page.
    pub data: Vec<Session>,
    /// Pagination token for the next page, when present.
    pub next_page: Option<String>,
    /// Pagination token for the previous page, when present.
    #[serde(default)]
    pub prev_page: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_agent_ref_id_serializes_as_bare_string() {
        assert_eq!(
            serde_json::to_value(SessionAgentRef::id("agent_abc")).unwrap(),
            "agent_abc"
        );
    }

    #[test]
    fn session_agent_ref_versioned_serializes_as_object() {
        let json = serde_json::to_value(SessionAgentRef::versioned("agent_abc", 4)).unwrap();
        assert_eq!(json["type"], "agent");
        assert_eq!(json["id"], "agent_abc");
        assert_eq!(json["version"], 4);
    }

    #[test]
    fn create_session_request_omits_unset_title() {
        let request = CreateSessionRequest::new(SessionAgentRef::id("agent_abc"), "env_abc");
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["agent"], "agent_abc");
        assert_eq!(json["environment_id"], "env_abc");
        assert!(json.get("title").is_none());
    }

    #[test]
    fn create_session_request_from_str_agent_shorthand() {
        let request = CreateSessionRequest::new("agent_abc", "env_abc").title("Hello");
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["agent"], "agent_abc");
        assert_eq!(json["title"], "Hello");
    }

    #[test]
    fn update_session_request_omits_unset_fields() {
        let request = UpdateSessionRequest::new().title("Renamed");
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["title"], "Renamed");
        assert!(json.get("metadata").is_none());
    }

    #[test]
    fn list_sessions_query_string_includes_order() {
        let query = ListSessionsParams::new()
            .limit(10)
            .order(SessionOrder::Desc)
            .to_query_string();
        assert!(query.contains("limit=10"));
        assert!(query.contains("order=desc"));
    }

    #[test]
    fn session_response_deserializes_with_full_object() {
        let session: Session = serde_json::from_value(serde_json::json!({
            "id": "sesn_01Abcdefghijklmnopqrstuvwx",
            "type": "session",
            "title": "Hello World Session",
            "status": "idle",
            "environment_id": "env_01Abcdefghijklmnopqrstuvwx",
            "created_at": "2026-03-15T10:00:00Z",
            "updated_at": "2026-03-15T10:00:00Z",
            "archived_at": null,
            "agent": {"id": "agent_abc", "version": 1},
            "resources": [],
            "metadata": {"foo": "bar"},
            "usage": {"input_tokens": 0, "output_tokens": 0}
        }))
        .unwrap();

        assert_eq!(session.id, "sesn_01Abcdefghijklmnopqrstuvwx");
        assert_eq!(session.status, SessionStatus::Idle);
        assert_eq!(session.metadata.get("foo"), Some(&"bar".to_string()));
        assert!(session.archived_at.is_none());
    }

    #[test]
    fn session_round_trips_through_serialize_and_deserialize() {
        let session: Session = serde_json::from_value(serde_json::json!({
            "id": "sesn_01Abcdefghijklmnopqrstuvwx",
            "type": "session",
            "title": null,
            "status": "terminated",
            "environment_id": "env_01Abcdefghijklmnopqrstuvwx",
            "created_at": "2026-03-15T10:00:00Z",
            "updated_at": "2026-03-15T10:00:00Z",
            "archived_at": "2026-03-16T10:00:00Z",
            "agent": {},
            "resources": [],
            "metadata": {},
            "usage": {}
        }))
        .unwrap();

        let json = serde_json::to_value(&session).unwrap();
        let reparsed: Session = serde_json::from_value(json).unwrap();
        assert_eq!(reparsed.id, session.id);
        assert_eq!(reparsed.status, session.status);
        assert_eq!(reparsed.archived_at, session.archived_at);
    }
}
