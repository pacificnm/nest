//! Low-level Claude Messages API client.

use nest_http::{HttpMethod, HttpRequest};
use nest_http_client::{HttpClientConfig, HttpClientService};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::agents::{
    Agent, AgentListPage, CreateAgentRequest, ListAgentsParams, UpdateAgentRequest,
};
use crate::config::ClaudeConfig;
use crate::count_tokens::{CountTokensRequest, TokenCountResponse};
use crate::error::{ClaudeError, ClaudeResult};
use crate::request::CreateMessageRequest;
use crate::response::MessageResponse;
use crate::sessions::{
    CreateSessionRequest, ListSessionsParams, Session, SessionDeleted, SessionListPage,
    UpdateSessionRequest,
};
use crate::skills::{
    encode_multipart_files, generate_boundary, FileUpload, ListSkillsParams, Skill, SkillDeleted,
    SkillListPage,
};
use crate::stream::MessageStream;

const MESSAGES_PATH: &str = "/v1/messages";
const COUNT_TOKENS_PATH: &str = "/v1/messages/count_tokens";
const SKILLS_PATH: &str = "/v1/skills";
const SKILLS_BETA_HEADER: &str = "skills-2025-10-02";
const AGENTS_PATH: &str = "/v1/agents";
const SESSIONS_PATH: &str = "/v1/sessions";
/// Beta header required by every Managed Agents endpoint (Agents, Sessions,
/// Environments, ...), not just Agents.
const MANAGED_AGENTS_BETA_HEADER: &str = "managed-agents-2026-04-01";

/// Claude (Anthropic) Messages API client.
///
/// Owns a dedicated [`HttpClientService`] configured with the `x-api-key` and
/// `anthropic-version` headers required on every request — it is not the
/// shared service registered by `nest-http-client`'s `HttpClientModule`,
/// since that service is process-wide and must not carry one module's API key.
#[derive(Clone)]
pub struct ClaudeClient {
    http: HttpClientService,
    config: ClaudeConfig,
}

impl ClaudeClient {
    /// Creates a client from resolved configuration.
    pub fn new(config: ClaudeConfig) -> ClaudeResult<Self> {
        let http_config = HttpClientConfig::default()
            .with_default_header("x-api-key", config.api_key.clone())
            .with_default_header("anthropic-version", config.anthropic_version.clone());
        let http = HttpClientService::new(http_config).map_err(ClaudeError::from)?;
        Ok(Self { http, config })
    }

    /// Returns the resolved configuration.
    pub fn config(&self) -> &ClaudeConfig {
        &self.config
    }

    /// Sends `POST /v1/messages` and returns the complete response.
    pub async fn create_message(
        &self,
        request: CreateMessageRequest,
    ) -> ClaudeResult<MessageResponse> {
        let body = request.into_body(&self.config, false);
        self.http
            .post_json(&self.messages_url(), &body)
            .await
            .map_err(ClaudeError::from)
    }

    /// Sends `POST /v1/messages` with `stream: true` and returns the event stream.
    pub async fn stream_message(
        &self,
        request: CreateMessageRequest,
    ) -> ClaudeResult<MessageStream> {
        let body = request.into_body(&self.config, true);
        let bytes = self
            .http
            .post_json_stream(&self.messages_url(), &body)
            .await
            .map_err(ClaudeError::from)?;
        Ok(MessageStream::new(bytes))
    }

    /// Sends `POST /v1/messages/count_tokens` and returns the token count.
    ///
    /// Does not create a message or consume `max_tokens`/rate-limit quota for
    /// output — only input tokens (messages, system prompt, tools) are counted.
    pub async fn count_tokens(
        &self,
        request: CountTokensRequest,
    ) -> ClaudeResult<TokenCountResponse> {
        let body = request.into_body(&self.config);
        self.http
            .post_json(&self.count_tokens_url(), &body)
            .await
            .map_err(ClaudeError::from)
    }

    /// Sends `POST /v1/skills`, uploading `files` to create a skill (and its
    /// first version) in one call.
    ///
    /// `files` should include a `SKILL.md` (and any supporting files) under a
    /// common top-level directory prefix, e.g. `"my-skill/SKILL.md"` — the API
    /// derives `display_title`/`directory`/`description` from that upload
    /// rather than from separate request fields.
    pub async fn create_skill(&self, files: Vec<FileUpload>) -> ClaudeResult<Skill> {
        let boundary = generate_boundary();
        let body = encode_multipart_files(&files, &boundary);
        let request = HttpRequest::post(self.skills_url())
            .with_header("anthropic-beta", SKILLS_BETA_HEADER)
            .with_header(
                "content-type",
                format!("multipart/form-data; boundary={boundary}"),
            )
            .with_body(body);
        self.send_json(request).await
    }

    /// Sends `GET /v1/skills` and returns one page of skills.
    pub async fn list_skills(&self, params: ListSkillsParams) -> ClaudeResult<SkillListPage> {
        let url = format!("{}{}", self.skills_url(), params.to_query_string());
        let request = HttpRequest::get(url).with_header("anthropic-beta", SKILLS_BETA_HEADER);
        self.send_json(request).await
    }

    /// Sends `GET /v1/skills/{skill_id}`.
    pub async fn get_skill(&self, skill_id: &str) -> ClaudeResult<Skill> {
        let request = HttpRequest::get(self.skill_url(skill_id))
            .with_header("anthropic-beta", SKILLS_BETA_HEADER);
        self.send_json(request).await
    }

    /// Sends `DELETE /v1/skills/{skill_id}`.
    pub async fn delete_skill(&self, skill_id: &str) -> ClaudeResult<SkillDeleted> {
        let request = HttpRequest::new(HttpMethod::Delete, self.skill_url(skill_id))
            .with_header("anthropic-beta", SKILLS_BETA_HEADER);
        self.send_json(request).await
    }

    /// Sends `POST /v1/agents` to create a persisted, versioned agent config.
    ///
    /// Store the returned [`Agent::id`] and reuse it — creating an agent is a
    /// setup step, not something to repeat per session/request.
    pub async fn create_agent(&self, request: CreateAgentRequest) -> ClaudeResult<Agent> {
        let body = self.json_body(&request)?;
        let http_request = HttpRequest::post(self.agents_url())
            .with_header("anthropic-beta", MANAGED_AGENTS_BETA_HEADER)
            .with_header("content-type", "application/json")
            .with_body(body);
        self.send_json(http_request).await
    }

    /// Sends `GET /v1/agents` and returns one page of agents.
    pub async fn list_agents(&self, params: ListAgentsParams) -> ClaudeResult<AgentListPage> {
        let url = format!("{}{}", self.agents_url(), params.to_query_string());
        let request =
            HttpRequest::get(url).with_header("anthropic-beta", MANAGED_AGENTS_BETA_HEADER);
        self.send_json(request).await
    }

    /// Sends `GET /v1/agents/{agent_id}`, optionally pinned to a specific `version`.
    pub async fn get_agent(&self, agent_id: &str, version: Option<u64>) -> ClaudeResult<Agent> {
        let mut url = self.agent_url(agent_id);
        if let Some(version) = version {
            url.push_str(&format!("?version={version}"));
        }
        let request =
            HttpRequest::get(url).with_header("anthropic-beta", MANAGED_AGENTS_BETA_HEADER);
        self.send_json(request).await
    }

    /// Sends `POST /v1/agents/{agent_id}` to update the agent, creating a new version.
    ///
    /// `request` must be pinned to the agent's current version (optimistic
    /// concurrency lock) — fetch it via [`ClaudeClient::get_agent`] first.
    pub async fn update_agent(
        &self,
        agent_id: &str,
        request: UpdateAgentRequest,
    ) -> ClaudeResult<Agent> {
        let body = self.json_body(&request)?;
        let http_request = HttpRequest::post(self.agent_url(agent_id))
            .with_header("anthropic-beta", MANAGED_AGENTS_BETA_HEADER)
            .with_header("content-type", "application/json")
            .with_body(body);
        self.send_json(http_request).await
    }

    /// Sends `POST /v1/agents/{agent_id}/archive`.
    ///
    /// Archiving is **permanent** — the agent becomes read-only and new
    /// sessions cannot reference it; there is no unarchive.
    pub async fn archive_agent(&self, agent_id: &str) -> ClaudeResult<Agent> {
        let url = format!("{}/archive", self.agent_url(agent_id));
        let request = HttpRequest::new(HttpMethod::Post, url)
            .with_header("anthropic-beta", MANAGED_AGENTS_BETA_HEADER);
        self.send_json(request).await
    }

    /// Sends `POST /v1/sessions` to start a session running a pre-created agent.
    ///
    /// A session is meaningless without an agent — create one via
    /// [`ClaudeClient::create_agent`] first, and an environment via a direct
    /// `POST /v1/environments` call (the Environments API isn't wrapped by
    /// this crate yet).
    pub async fn create_session(&self, request: CreateSessionRequest) -> ClaudeResult<Session> {
        let body = self.json_body(&request)?;
        let http_request = HttpRequest::post(self.sessions_url())
            .with_header("anthropic-beta", MANAGED_AGENTS_BETA_HEADER)
            .with_header("content-type", "application/json")
            .with_body(body);
        self.send_json(http_request).await
    }

    /// Sends `GET /v1/sessions` and returns one page of sessions.
    ///
    /// Sessions is the one Managed Agents list endpoint that supports
    /// backward pagination — see [`SessionListPage::prev_page`].
    pub async fn list_sessions(&self, params: ListSessionsParams) -> ClaudeResult<SessionListPage> {
        let url = format!("{}{}", self.sessions_url(), params.to_query_string());
        let request =
            HttpRequest::get(url).with_header("anthropic-beta", MANAGED_AGENTS_BETA_HEADER);
        self.send_json(request).await
    }

    /// Sends `GET /v1/sessions/{session_id}`.
    pub async fn get_session(&self, session_id: &str) -> ClaudeResult<Session> {
        let request = HttpRequest::get(self.session_url(session_id))
            .with_header("anthropic-beta", MANAGED_AGENTS_BETA_HEADER);
        self.send_json(request).await
    }

    /// Sends `POST /v1/sessions/{session_id}` to update `title`/`metadata`.
    pub async fn update_session(
        &self,
        session_id: &str,
        request: UpdateSessionRequest,
    ) -> ClaudeResult<Session> {
        let body = self.json_body(&request)?;
        let http_request = HttpRequest::post(self.session_url(session_id))
            .with_header("anthropic-beta", MANAGED_AGENTS_BETA_HEADER)
            .with_header("content-type", "application/json")
            .with_body(body);
        self.send_json(http_request).await
    }

    /// Sends `DELETE /v1/sessions/{session_id}`.
    ///
    /// Permanently deletes the session, its event history, container, and
    /// checkpoints. Use [`ClaudeClient::archive_session`] instead if you want
    /// to keep the session inspectable.
    pub async fn delete_session(&self, session_id: &str) -> ClaudeResult<SessionDeleted> {
        let request = HttpRequest::new(HttpMethod::Delete, self.session_url(session_id))
            .with_header("anthropic-beta", MANAGED_AGENTS_BETA_HEADER);
        self.send_json(request).await
    }

    /// Sends `POST /v1/sessions/{session_id}/archive`.
    ///
    /// Archiving is **permanent** — the session becomes read-only; there is
    /// no unarchive. Use [`ClaudeClient::delete_session`] instead if you want
    /// the session (and its event history) gone entirely.
    pub async fn archive_session(&self, session_id: &str) -> ClaudeResult<Session> {
        let url = format!("{}/archive", self.session_url(session_id));
        let request = HttpRequest::new(HttpMethod::Post, url)
            .with_header("anthropic-beta", MANAGED_AGENTS_BETA_HEADER);
        self.send_json(request).await
    }

    fn messages_url(&self) -> String {
        format!("{}{}", self.config.base_url, MESSAGES_PATH)
    }

    fn count_tokens_url(&self) -> String {
        format!("{}{}", self.config.base_url, COUNT_TOKENS_PATH)
    }

    fn skills_url(&self) -> String {
        format!("{}{}", self.config.base_url, SKILLS_PATH)
    }

    fn skill_url(&self, skill_id: &str) -> String {
        format!("{}/{}", self.skills_url(), skill_id)
    }

    fn agents_url(&self) -> String {
        format!("{}{}", self.config.base_url, AGENTS_PATH)
    }

    fn agent_url(&self, agent_id: &str) -> String {
        format!("{}/{}", self.agents_url(), agent_id)
    }

    fn sessions_url(&self) -> String {
        format!("{}{}", self.config.base_url, SESSIONS_PATH)
    }

    fn session_url(&self, session_id: &str) -> String {
        format!("{}/{}", self.sessions_url(), session_id)
    }

    /// Sends a fully-built request and decodes a JSON response — used for
    /// endpoints (e.g. Skills, Agents) that need headers or a body shape the
    /// `post_json`/`get_json` convenience methods don't support.
    async fn send_json<T: DeserializeOwned>(&self, request: HttpRequest) -> ClaudeResult<T> {
        let response = self.http.send(request).await.map_err(ClaudeError::from)?;
        serde_json::from_slice(&response.body)
            .map_err(|error| ClaudeError::parse(format!("failed to decode JSON: {error}")))
    }

    /// Encodes a request body to JSON bytes for use with [`Self::send_json`].
    fn json_body(&self, body: &impl Serialize) -> ClaudeResult<Vec<u8>> {
        serde_json::to_vec(body)
            .map_err(|error| ClaudeError::parse(format!("failed to encode request body: {error}")))
    }
}

#[cfg(test)]
mod tests {
    use futures_util::StreamExt;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;
    use crate::request::Message;
    use crate::stream::StreamEvent;
    use crate::types::StopReason;

    fn test_config(base_uri: &str) -> ClaudeConfig {
        ClaudeConfig::builder()
            .api_key("test-key")
            .base_url(base_uri)
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn create_message_sends_headers_and_parses_response() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .and(header("x-api-key", "test-key"))
            .and(header("anthropic-version", "2023-06-01"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "msg_1",
                "type": "message",
                "role": "assistant",
                "content": [{"type": "text", "text": "Hello!"}],
                "model": "claude-opus-4-8",
                "stop_reason": "end_turn",
                "stop_sequence": null,
                "usage": {"input_tokens": 8, "output_tokens": 3}
            })))
            .mount(&server)
            .await;

        let client = ClaudeClient::new(test_config(&server.uri())).unwrap();
        let response = client
            .create_message(CreateMessageRequest::new(vec![Message::user("hi")]))
            .await
            .unwrap();

        assert_eq!(response.text(), "Hello!");
        assert_eq!(response.stop_reason, Some(StopReason::EndTurn));
    }

    #[tokio::test]
    async fn create_message_maps_400_to_invalid_request() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(ResponseTemplate::new(400).set_body_json(serde_json::json!({
                "type": "error",
                "error": {"type": "invalid_request_error", "message": "max_tokens is required"}
            })))
            .mount(&server)
            .await;

        let client = ClaudeClient::new(test_config(&server.uri())).unwrap();
        let error = client
            .create_message(CreateMessageRequest::new(vec![Message::user("hi")]))
            .await
            .unwrap_err();

        assert_eq!(
            error.code(),
            Some(crate::codes::NEST_CLAUDE_INVALID_REQUEST)
        );
    }

    #[tokio::test]
    async fn create_message_maps_429_to_rate_limited() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(ResponseTemplate::new(429).set_body_json(serde_json::json!({
                "type": "error",
                "error": {"type": "rate_limit_error", "message": "slow down"}
            })))
            .mount(&server)
            .await;

        let client = ClaudeClient::new(test_config(&server.uri())).unwrap();
        let error = client
            .create_message(CreateMessageRequest::new(vec![Message::user("hi")]))
            .await
            .unwrap_err();

        assert_eq!(error.code(), Some(crate::codes::NEST_CLAUDE_RATE_LIMITED));
    }

    #[tokio::test]
    async fn stream_message_yields_parsed_events() {
        let server = MockServer::start().await;
        let sse = concat!(
            "event: content_block_delta\n",
            "data: {\"type\":\"content_block_delta\",\"index\":0,",
            "\"delta\":{\"type\":\"text_delta\",\"text\":\"Hi\"}}\n\n",
            "event: message_stop\n",
            "data: {\"type\":\"message_stop\"}\n\n",
        );
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(sse, "text/event-stream"))
            .mount(&server)
            .await;

        let client = ClaudeClient::new(test_config(&server.uri())).unwrap();
        let mut stream = client
            .stream_message(CreateMessageRequest::new(vec![Message::user("hi")]))
            .await
            .unwrap();

        let first = stream.next().await.unwrap().unwrap();
        assert!(matches!(first, StreamEvent::ContentBlockDelta { .. }));
        let second = stream.next().await.unwrap().unwrap();
        assert!(matches!(second, StreamEvent::MessageStop));
        assert!(stream.next().await.is_none());
    }

    #[tokio::test]
    async fn create_message_body_marks_stream_false() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/messages"))
            .and(body_json_string_contains("\"stream\":false"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "msg_1",
                "type": "message",
                "role": "assistant",
                "content": [{"type": "text", "text": "ok"}],
                "model": "claude-opus-4-8",
                "stop_reason": "end_turn",
                "stop_sequence": null,
                "usage": {"input_tokens": 1, "output_tokens": 1}
            })))
            .mount(&server)
            .await;

        let client = ClaudeClient::new(test_config(&server.uri())).unwrap();
        client
            .create_message(CreateMessageRequest::new(vec![Message::user("hi")]))
            .await
            .unwrap();
    }

    fn body_json_string_contains(needle: &'static str) -> impl wiremock::Match {
        struct Contains(&'static str);
        impl wiremock::Match for Contains {
            fn matches(&self, request: &wiremock::Request) -> bool {
                String::from_utf8_lossy(&request.body).contains(self.0)
            }
        }
        Contains(needle)
    }

    #[tokio::test]
    async fn count_tokens_posts_to_dedicated_path_and_parses_response() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/messages/count_tokens"))
            .and(header("x-api-key", "test-key"))
            .and(header("anthropic-version", "2023-06-01"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "input_tokens": 2095
            })))
            .mount(&server)
            .await;

        let client = ClaudeClient::new(test_config(&server.uri())).unwrap();
        let response = client
            .count_tokens(CountTokensRequest::new(vec![Message::user("Hello, world")]))
            .await
            .unwrap();

        assert_eq!(response.input_tokens, 2095);
        assert!(response.context_management.is_none());
    }

    #[tokio::test]
    async fn count_tokens_body_omits_max_tokens_and_stream() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/messages/count_tokens"))
            .and(body_json_string_contains("\"model\""))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "input_tokens": 1
            })))
            .mount(&server)
            .await;

        let client = ClaudeClient::new(test_config(&server.uri())).unwrap();
        client
            .count_tokens(CountTokensRequest::new(vec![Message::user("hi")]))
            .await
            .unwrap();
    }

    fn skill_json() -> serde_json::Value {
        serde_json::json!({
            "id": "skill_01JAbcdefghijklmnopqrstuvw",
            "created_at": "2024-10-30T23:58:27.427722Z",
            "updated_at": "2024-10-30T23:58:27.427722Z",
            "display_title": "My Custom Skill",
            "latest_version": "1759178010641129",
            "source": "custom"
        })
    }

    #[tokio::test]
    async fn create_skill_uploads_multipart_and_parses_response() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/skills"))
            .and(header("anthropic-beta", "skills-2025-10-02"))
            .and(body_json_string_contains("name=\"files[]\""))
            .and(body_json_string_contains("SKILL.md"))
            .respond_with(ResponseTemplate::new(200).set_body_json(skill_json()))
            .mount(&server)
            .await;

        let client = ClaudeClient::new(test_config(&server.uri())).unwrap();
        let skill = client
            .create_skill(vec![crate::skills::FileUpload::new(
                "my-skill/SKILL.md",
                b"---\nname: my-skill\n---\nBody".to_vec(),
            )])
            .await
            .unwrap();

        assert_eq!(skill.display_title, "My Custom Skill");
        assert_eq!(skill.source, crate::skills::SkillSource::Custom);
    }

    #[tokio::test]
    async fn list_skills_sends_query_params_and_parses_page() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/skills"))
            .and(header("anthropic-beta", "skills-2025-10-02"))
            .and(wiremock::matchers::query_param("limit", "50"))
            .and(wiremock::matchers::query_param("source", "custom"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [skill_json()],
                "has_more": true,
                "next_page": "page_abc"
            })))
            .mount(&server)
            .await;

        let client = ClaudeClient::new(test_config(&server.uri())).unwrap();
        let page = client
            .list_skills(
                crate::skills::ListSkillsParams::new()
                    .limit(50)
                    .source(crate::skills::SkillSource::Custom),
            )
            .await
            .unwrap();

        assert_eq!(page.data.len(), 1);
        assert!(page.has_more);
        assert_eq!(page.next_page.as_deref(), Some("page_abc"));
    }

    #[tokio::test]
    async fn get_skill_returns_skill() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/skills/skill_01JAbcdefghijklmnopqrstuvw"))
            .and(header("anthropic-beta", "skills-2025-10-02"))
            .respond_with(ResponseTemplate::new(200).set_body_json(skill_json()))
            .mount(&server)
            .await;

        let client = ClaudeClient::new(test_config(&server.uri())).unwrap();
        let skill = client
            .get_skill("skill_01JAbcdefghijklmnopqrstuvw")
            .await
            .unwrap();

        assert_eq!(skill.id, "skill_01JAbcdefghijklmnopqrstuvw");
    }

    #[tokio::test]
    async fn delete_skill_returns_deleted_id() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/v1/skills/skill_01JAbcdefghijklmnopqrstuvw"))
            .and(header("anthropic-beta", "skills-2025-10-02"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "skill_01JAbcdefghijklmnopqrstuvw",
                "type": "skill_deleted"
            })))
            .mount(&server)
            .await;

        let client = ClaudeClient::new(test_config(&server.uri())).unwrap();
        let deleted = client
            .delete_skill("skill_01JAbcdefghijklmnopqrstuvw")
            .await
            .unwrap();

        assert_eq!(deleted.id, "skill_01JAbcdefghijklmnopqrstuvw");
    }

    fn agent_json(version: u64, archived_at: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "id": "agent_011CZkYpogX7uDKUyvBTophP",
            "type": "agent",
            "version": version,
            "name": "My First Agent",
            "description": "A general-purpose starter agent.",
            "created_at": "2026-03-15T10:00:00Z",
            "updated_at": "2026-03-15T10:00:00Z",
            "archived_at": archived_at,
            "model": {"id": "claude-sonnet-4-6", "speed": "standard"},
            "system": "You are a general-purpose agent.",
            "tools": [{"type": "agent_toolset_20260401"}],
            "skills": [],
            "mcp_servers": [],
            "metadata": {}
        })
    }

    #[tokio::test]
    async fn create_agent_sends_beta_header_and_parses_response() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/agents"))
            .and(header("anthropic-beta", "managed-agents-2026-04-01"))
            .and(body_json_string_contains("\"name\":\"My First Agent\""))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(agent_json(1, serde_json::Value::Null)),
            )
            .mount(&server)
            .await;

        let client = ClaudeClient::new(test_config(&server.uri())).unwrap();
        let agent = client
            .create_agent(crate::agents::CreateAgentRequest::new(
                "My First Agent",
                "claude-sonnet-4-6",
            ))
            .await
            .unwrap();

        assert_eq!(agent.id, "agent_011CZkYpogX7uDKUyvBTophP");
        assert_eq!(agent.version, 1);
    }

    #[tokio::test]
    async fn list_agents_sends_query_params_and_parses_page() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/agents"))
            .and(header("anthropic-beta", "managed-agents-2026-04-01"))
            .and(wiremock::matchers::query_param("include_archived", "true"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [agent_json(1, serde_json::Value::Null)],
                "next_page": null
            })))
            .mount(&server)
            .await;

        let client = ClaudeClient::new(test_config(&server.uri())).unwrap();
        let page = client
            .list_agents(crate::agents::ListAgentsParams::new().include_archived(true))
            .await
            .unwrap();

        assert_eq!(page.data.len(), 1);
        assert!(page.next_page.is_none());
    }

    #[tokio::test]
    async fn get_agent_sends_version_query_param() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/agents/agent_011CZkYpogX7uDKUyvBTophP"))
            .and(header("anthropic-beta", "managed-agents-2026-04-01"))
            .and(wiremock::matchers::query_param("version", "3"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(agent_json(3, serde_json::Value::Null)),
            )
            .mount(&server)
            .await;

        let client = ClaudeClient::new(test_config(&server.uri())).unwrap();
        let agent = client
            .get_agent("agent_011CZkYpogX7uDKUyvBTophP", Some(3))
            .await
            .unwrap();

        assert_eq!(agent.version, 3);
    }

    #[tokio::test]
    async fn update_agent_sends_version_lock_and_parses_response() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/agents/agent_011CZkYpogX7uDKUyvBTophP"))
            .and(header("anthropic-beta", "managed-agents-2026-04-01"))
            .and(body_json_string_contains("\"version\":1"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(agent_json(2, serde_json::Value::Null)),
            )
            .mount(&server)
            .await;

        let client = ClaudeClient::new(test_config(&server.uri())).unwrap();
        let agent = client
            .update_agent(
                "agent_011CZkYpogX7uDKUyvBTophP",
                crate::agents::UpdateAgentRequest::new(1).system("Updated prompt"),
            )
            .await
            .unwrap();

        assert_eq!(agent.version, 2);
    }

    #[tokio::test]
    async fn archive_agent_posts_to_archive_path() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/agents/agent_011CZkYpogX7uDKUyvBTophP/archive"))
            .and(header("anthropic-beta", "managed-agents-2026-04-01"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(agent_json(1, serde_json::json!("2026-03-16T10:00:00Z"))),
            )
            .mount(&server)
            .await;

        let client = ClaudeClient::new(test_config(&server.uri())).unwrap();
        let agent = client
            .archive_agent("agent_011CZkYpogX7uDKUyvBTophP")
            .await
            .unwrap();

        assert_eq!(agent.archived_at.as_deref(), Some("2026-03-16T10:00:00Z"));
    }

    fn session_json(status: &str, archived_at: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "id": "sesn_01Abcdefghijklmnopqrstuvwx",
            "type": "session",
            "title": "Hello World Session",
            "status": status,
            "environment_id": "env_01Abcdefghijklmnopqrstuvwx",
            "created_at": "2026-03-15T10:00:00Z",
            "updated_at": "2026-03-15T10:00:00Z",
            "archived_at": archived_at,
            "agent": {"id": "agent_011CZkYpogX7uDKUyvBTophP", "version": 1},
            "resources": [],
            "metadata": {},
            "usage": {}
        })
    }

    #[tokio::test]
    async fn create_session_sends_beta_header_and_parses_response() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/sessions"))
            .and(header("anthropic-beta", "managed-agents-2026-04-01"))
            .and(body_json_string_contains(
                "\"agent\":\"agent_011CZkYpogX7uDKUyvBTophP\"",
            ))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(session_json("idle", serde_json::Value::Null)),
            )
            .mount(&server)
            .await;

        let client = ClaudeClient::new(test_config(&server.uri())).unwrap();
        let session = client
            .create_session(crate::sessions::CreateSessionRequest::new(
                "agent_011CZkYpogX7uDKUyvBTophP",
                "env_01Abcdefghijklmnopqrstuvwx",
            ))
            .await
            .unwrap();

        assert_eq!(session.id, "sesn_01Abcdefghijklmnopqrstuvwx");
        assert_eq!(session.status, crate::sessions::SessionStatus::Idle);
    }

    #[tokio::test]
    async fn list_sessions_sends_query_params_and_parses_page() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/sessions"))
            .and(header("anthropic-beta", "managed-agents-2026-04-01"))
            .and(wiremock::matchers::query_param("order", "desc"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": [session_json("idle", serde_json::Value::Null)],
                "next_page": null,
                "prev_page": null
            })))
            .mount(&server)
            .await;

        let client = ClaudeClient::new(test_config(&server.uri())).unwrap();
        let page = client
            .list_sessions(
                crate::sessions::ListSessionsParams::new()
                    .order(crate::sessions::SessionOrder::Desc),
            )
            .await
            .unwrap();

        assert_eq!(page.data.len(), 1);
        assert!(page.next_page.is_none());
        assert!(page.prev_page.is_none());
    }

    #[tokio::test]
    async fn get_session_returns_session() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/sessions/sesn_01Abcdefghijklmnopqrstuvwx"))
            .and(header("anthropic-beta", "managed-agents-2026-04-01"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(session_json("running", serde_json::Value::Null)),
            )
            .mount(&server)
            .await;

        let client = ClaudeClient::new(test_config(&server.uri())).unwrap();
        let session = client
            .get_session("sesn_01Abcdefghijklmnopqrstuvwx")
            .await
            .unwrap();

        assert_eq!(session.status, crate::sessions::SessionStatus::Running);
    }

    #[tokio::test]
    async fn update_session_posts_title_and_parses_response() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/sessions/sesn_01Abcdefghijklmnopqrstuvwx"))
            .and(header("anthropic-beta", "managed-agents-2026-04-01"))
            .and(body_json_string_contains("\"title\":\"Renamed\""))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(session_json("idle", serde_json::Value::Null)),
            )
            .mount(&server)
            .await;

        let client = ClaudeClient::new(test_config(&server.uri())).unwrap();
        let session = client
            .update_session(
                "sesn_01Abcdefghijklmnopqrstuvwx",
                crate::sessions::UpdateSessionRequest::new().title("Renamed"),
            )
            .await
            .unwrap();

        assert_eq!(session.id, "sesn_01Abcdefghijklmnopqrstuvwx");
    }

    #[tokio::test]
    async fn delete_session_returns_deleted_id() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/v1/sessions/sesn_01Abcdefghijklmnopqrstuvwx"))
            .and(header("anthropic-beta", "managed-agents-2026-04-01"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "sesn_01Abcdefghijklmnopqrstuvwx",
                "type": "session_deleted"
            })))
            .mount(&server)
            .await;

        let client = ClaudeClient::new(test_config(&server.uri())).unwrap();
        let deleted = client
            .delete_session("sesn_01Abcdefghijklmnopqrstuvwx")
            .await
            .unwrap();

        assert_eq!(deleted.id, "sesn_01Abcdefghijklmnopqrstuvwx");
    }

    #[tokio::test]
    async fn archive_session_posts_to_archive_path() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/sessions/sesn_01Abcdefghijklmnopqrstuvwx/archive"))
            .and(header("anthropic-beta", "managed-agents-2026-04-01"))
            .respond_with(ResponseTemplate::new(200).set_body_json(session_json(
                "terminated",
                serde_json::json!("2026-03-16T10:00:00Z"),
            )))
            .mount(&server)
            .await;

        let client = ClaudeClient::new(test_config(&server.uri())).unwrap();
        let session = client
            .archive_session("sesn_01Abcdefghijklmnopqrstuvwx")
            .await
            .unwrap();

        assert_eq!(session.archived_at.as_deref(), Some("2026-03-16T10:00:00Z"));
    }
}
