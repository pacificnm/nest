# nest-claude v1 Implementation Plan

## Status: Implemented

## Context

`nest-claude` gives Nest apps a typed client for the Claude (Anthropic) Messages API (`POST /v1/messages`) — the single endpoint that covers plain completion, tool use, extended thinking, and streaming per the [Claude API docs](https://platform.claude.com/docs/en/api/overview).

**Design principle:** `nest-claude` is a **provider adapter**, same shape as `nest-tmdb`/`nest-airtable`. It owns Claude HTTP calls, request/response DTOs, SSE parsing, and error mapping. It does not implement an agent loop, MCP, or any UI.

## Crate boundaries

| Crate | Layer | Role |
|-------|-------|------|
| `nest-http-client` | **core** | `HttpClientService`/`HttpClientConfig` — `nest-claude` builds its **own** instance rather than reusing the app-wide shared one |
| `nest-error` | **core** | `NestError`/`NestResult` |
| **`nest-claude`** | **module** | Claude Messages API client, request/response types, SSE stream parsing |

### Why a dedicated `HttpClientService` instead of the shared one

`nest-airtable`/`nest-tmdb` reuse the app-wide `HttpClientService` registered by `HttpClientModule` and inject auth per-request (query param or header) since that service has no fixed auth. `nest-claude` needs `x-api-key` + `anthropic-version` on every request, including streaming (`post_json_stream`, which applies `HttpClientConfig::default_headers` but takes no per-call header argument). Setting those as `default_headers` on the *shared* service would leak the Claude API key into every other module's requests through that same `HttpClientService` instance. `ClaudeClient::new` therefore constructs its own `HttpClientService::new(HttpClientConfig::default().with_default_header(...))` — isolated, so `ClaudeModule` has no dependency on `HTTP_CLIENT_MODULE_ID`.

### Hard boundaries

`nest-claude` **must not**:

- Depend on `nest-logging` (feature crates emit `tracing::` events only — see [`nest-logging` overview](../nest-logging/overview.md))
- Implement an agentic tool-call loop, MCP, or Managed Agents (that belongs in an app or a future `nest-agent`/`nest-mcp` integration)
- Mutate the shared `HttpClientService` registered by `HttpClientModule`

## Endpoints (added one at a time)

| Endpoint | Client method | Status |
|---|---|---|
| `POST /v1/messages` (non-streaming) | `ClaudeClient::create_message` | Implemented |
| `POST /v1/messages` (streaming, `stream: true`) | `ClaudeClient::stream_message` | Implemented |
| `POST /v1/messages/count_tokens` | `ClaudeClient::count_tokens` | Implemented |
| `POST /v1/skills` (Create Skill) | `ClaudeClient::create_skill` | Implemented |
| `GET /v1/skills` (List Skills) | `ClaudeClient::list_skills` | Implemented |
| `GET /v1/skills/{skill_id}` (Get Skill) | `ClaudeClient::get_skill` | Implemented |
| `DELETE /v1/skills/{skill_id}` (Delete Skill) | `ClaudeClient::delete_skill` | Implemented |
| `POST/GET/DELETE /v1/skills/{skill_id}/versions[/{version}]`, content download | — | Not started |
| `POST /v1/agents` (Create Agent) | `ClaudeClient::create_agent` | Implemented |
| `GET /v1/agents` (List Agents) | `ClaudeClient::list_agents` | Implemented |
| `GET /v1/agents/{agent_id}` (Get Agent) | `ClaudeClient::get_agent` | Implemented |
| `POST /v1/agents/{agent_id}` (Update Agent) | `ClaudeClient::update_agent` | Implemented |
| `POST /v1/agents/{agent_id}/archive` (Archive Agent) | `ClaudeClient::archive_agent` | Implemented |
| `GET /v1/agents/{agent_id}/versions` (List Agent Versions) | — | Not started |
| `POST /v1/sessions` (Create Session) | `ClaudeClient::create_session` | Implemented |
| `GET /v1/sessions` (List Sessions) | `ClaudeClient::list_sessions` | Implemented |
| `GET /v1/sessions/{session_id}` (Get Session) | `ClaudeClient::get_session` | Implemented |
| `POST /v1/sessions/{session_id}` (Update Session) | `ClaudeClient::update_session` | Implemented |
| `DELETE /v1/sessions/{session_id}` (Delete Session) | `ClaudeClient::delete_session` | Implemented |
| `POST /v1/sessions/{session_id}/archive` (Archive Session) | `ClaudeClient::archive_session` | Implemented |
| Session Events (send/list/stream) | — | Not started |
| Managed Agents environments/vaults/deployments | — | Not started |
| `POST /v1/messages/batches` + friends | — | Not started |
| `POST /v1/files` + friends | — | Not started |
| `GET /v1/models`, `GET /v1/models/{id}` | — | Not started |

## v1 scope

Covered, in addition to the endpoints above:

- Content blocks: text, image (base64/URL), `tool_use`, `tool_result`, `thinking`/`redacted_thinking` (echo-back)
- Tool use: `ToolDefinition`, `ToolChoice`
- Extended (adaptive) thinking + `output_config.effort`
- Prompt caching: `cache_control` (ephemeral, 5m/1h TTL) on text blocks, tool definitions, and top-level
- Config via explicit builder, `ANTHROPIC_API_KEY` env, or `[claude]` in `ConfigService` (matches `nest-tmdb`'s `#[cfg(feature = "config")]` pattern)
- Nest module registration (`ClaudeModule`) for DI-based apps

Deferred to a later version (flagged in crate/README docs so callers don't assume support):

- Files API / PDF & document content blocks
- Server-side tools (web search, web fetch, code execution)
- Message Batches API
- Models API
- Skill versions (`/v1/skills/{id}/versions`) and content download
- Structured outputs (`output_config.format`)
- Managed Agents environments/vaults/deployments (created ad hoc, via raw HTTP, only where a session example needs one), Session Events (send/list/stream — a created session can't yet hold a live conversation), and agent/session versions listing

### Skills-specific notes

- Every Skills endpoint requires `anthropic-beta: skills-2025-10-02`. Unlike `x-api-key`/`anthropic-version`, this is **not** a default header on `ClaudeClient`'s `HttpClientService` — it's added per-request in `client.rs`, since it doesn't apply to `/v1/messages` or count_tokens and shouldn't be sent there.
- `create_skill` is a `multipart/form-data` POST — the first Nest-claude endpoint that isn't plain JSON. `post_json`/`get_json`/`post_json_stream` on `HttpClientService` only support JSON bodies and don't accept per-call headers, so Skills calls go through a new `ClaudeClient::send_json` helper built on the lower-level `HttpClientService::send(HttpRequest)` — same idiom `nest-airtable` uses for its `authorized()`/`send_airtable()` per-request header injection.
- `nest-claude` hand-rolls the multipart encoder (`skills::encode_multipart_files`, boundary via `SystemTime` nanos) rather than adding a `multipart`/`reqwest::multipart` dependency, since it's one field (`files[]`, repeatable) and `nest-http-client`'s `HttpRequest` only carries raw `Vec<u8>` bodies anyway (no passthrough to reqwest's multipart builder).
- The API derives `display_title`/`directory`/`description` from the uploaded `SKILL.md`, not from separate request fields — `create_skill` takes only `Vec<FileUpload>`.
- **Live-API correction:** the field name is `files[]`, not `files` — the API's own example curl (`-F files=...`) is wrong/misleading; a bare `files` field 400s with `files[]: Field required`. Caught by running `examples/create_nest_core_skill.rs` against the real API (wiremock tests alone didn't catch it, since they only assert the mock's own expectations, not Anthropic's real validation). Fixed in `encode_multipart_files` + the wiremock assertion + unit test.

### Agents-specific notes

- Every Agents endpoint requires `anthropic-beta: managed-agents-2026-04-01`, sent per-request (same pattern as the Skills beta header) via the `create_agent`/`list_agents`/`get_agent`/`update_agent`/`archive_agent` methods, all routed through `send_json` + a small `json_body` encoding helper (also new this turn — Agents needed a JSON POST with a custom header, which `post_json` doesn't support any more than it supports Skills' multipart).
- `model` is `string | {id, speed}` on the wire but always the object form in responses — modeled as two types: `AgentModel` (request-side, `#[serde(untagged)]`, `From<&str>`/`From<String>` for the common bare-ID case) vs `AgentModelInfo` (response-side, always `{id, speed}`).
- `AgentRef` (a multiagent roster entry: bare agent-id string | `{type:"agent",id,version}` | `{type:"self"}`) needed a **hand-written** `Serialize`/`Deserialize` impl (via an intermediate `serde_json::Value`) rather than a derived one — `#[serde(untagged)]` can't also emit `{"type":"self"}` for a fieldless variant (it serializes untagged unit variants as `null`), and `#[serde(tag = "type")]` can't express the bare-string variant. This is the only hand-rolled serde impl in the crate so far; every other tagged union here fits `#[serde(tag = "type")]` cleanly.
- `UpdateAgentRequest` matches the API's actual patch semantics: omitting a field (`None`) preserves it; `description`/`system` are cleared by sending an empty string (`clear_description()`/`clear_system()`); `tools`/`skills`/`mcp_servers` are cleared by sending an empty array (`clear_tools()` etc., full-replacement otherwise — no diffing); `metadata` is `HashMap<String, Option<String>>` where `Some(v)` upserts a key and `None` deletes it. There's no `None`-vs-omitted distinction available for `description`/`system` (both collapse to Rust's `None`), so "send explicit null to clear" isn't separately modeled — the documented empty-string/empty-array clear path covers it.
- `ListAgentsParams` percent-encodes bracket-style filter keys (`created_at[gte]`, `created_at[lte]`) via a small shared `util::percent_encode` — pulled out of `skills.rs` (which had its own copy for the `page` token) into `src/util.rs` since it's now used by two modules.
- Sessions aren't covered, so an agent created via this crate can't yet be run — `create_agent`/`update_agent`/`archive_agent` manage the resource only.

### Sessions-specific notes

- Reuses `MANAGED_AGENTS_BETA_HEADER` (renamed from the Agents-only `AGENTS_BETA_HEADER`, since it's genuinely shared across both endpoint groups now) — same `managed-agents-2026-04-01` value, same per-request `send_json`/`json_body` routing as Agents.
- `SessionAgentRef` (the session's `agent` field on create) mirrors `AgentRef`'s bare-string-vs-tagged-object split, minus the `self` variant (which only makes sense inside a multiagent coordinator's roster, not a session) — a hand-written `Serialize` only, since there's no need to deserialize a reference back (`Session.agent` in a response is the *resolved* config, not a reference, and is kept as an untyped `serde_json::Value`).
- v1 deliberately excludes two related, more advanced features so they can be designed together later: `agent_with_overrides` (session-local override of model/system/tools/mcp_servers/skills) on create, and the mid-session `agent.tools`/`agent.mcp_servers`/`vault_ids` override on update (only valid while `idle`) — `UpdateSessionRequest` supports `title`/`metadata` only.
- Sessions is the one Managed Agents list endpoint with backward pagination — `SessionListPage` carries `prev_page` in addition to `next_page` (`AgentListPage`/`SkillListPage` don't).
- **A session requires an `environment_id`, and the Environments API isn't wrapped by this crate** — building it was out of scope for this pass. `examples/manage_nest_agent_sessions.rs` gets one via two raw `nest_http_client`/`HttpClientService` calls (`GET`/`POST /v1/environments`, find-by-name-or-create) rather than adding a full `environments.rs` module just to unblock testing Sessions.
- **Live-verified:** `create_session`/`get_session`/`list_sessions`/`update_session`/`archive_session`/`delete_session` all round-tripped against the real API in one run, including archiving and then deleting the *same* session back-to-back with no conflict — despite Agents having no delete (archive-only), Sessions' archive and delete turned out to compose fine. No messages were sent, so the session stayed `idle` throughout and no model inference was billed.

## Key types

- `ClaudeConfig` / `ClaudeConfigBuilder` — api key, base URL, `anthropic-version`, default model/max_tokens
- `ClaudeClient` — owns the dedicated `HttpClientService`; `create_message` / `stream_message`
- `CreateMessageRequest` — builder; resolves `model`/`max_tokens` from `ClaudeConfig` when unset via `into_body()`
- `Message`, `ContentBlock`, `SystemPrompt`, `ToolDefinition`, `ToolChoice`, `ThinkingConfig`, `OutputConfig`/`Effort`, `CacheControl`
- `MessageResponse` — `.text()` and `.tool_uses()` convenience accessors
- `StreamEvent`/`ContentDelta` — typed SSE events; `MessageStream` skips `ping` events and joins multi-line `data:` fields
- `CountTokensRequest`/`TokenCountResponse`/`ContextManagementTokenInfo` — mirrors `CreateMessageRequest` minus `max_tokens`/`stream`
- `FileUpload`, `Skill`/`SkillSource`, `SkillListPage`, `SkillDeleted`, `ListSkillsParams` — Skills API types
- `Agent`/`AgentModelInfo`, `CreateAgentRequest`/`UpdateAgentRequest`, `AgentModel`, `AgentTool`/`ToolConfig`/`NamedToolConfig`/`PermissionPolicy`, `AgentSkillRef`, `McpServerDefinition`, `Multiagent`/`AgentRef`, `ListAgentsParams`/`AgentListPage` — Agents API types
- `Session`/`SessionStatus`, `SessionAgentRef`, `CreateSessionRequest`/`UpdateSessionRequest`, `SessionDeleted`, `ListSessionsParams`/`SessionOrder`/`SessionListPage` — Sessions API types
- `ClaudeError`/`ClaudeErrorKind` — maps HTTP 400/401/403/429/5xx to `InvalidRequest`/`Auth`/`RateLimit`/`Server`; converts to `NestError`

## Verification

- `cargo test -p nest-claude` — 68 unit tests (config, error mapping, request body resolution, response parsing, SSE parsing incl. chunk-boundary splitting, module registration, token counting, Skills query-string/multipart encoding, Agents model/tool/skill/ref serde shapes incl. the hand-rolled `AgentRef` impl, update-patch clear/metadata semantics, Sessions request/response serde shapes incl. the hand-rolled `SessionAgentRef` impl and all six client methods) + 3 doc-tests, using `wiremock` for HTTP mocking
- `cargo clippy -p nest-claude --all-targets --examples` with `RUSTFLAGS=-D warnings` — clean
- `cargo check --workspace` — clean
- **Live API verification** (`examples/create_nest_core_skill.rs`, `examples/list_skills.rs`, `examples/create_nest_agent.rs`, `examples/list_agents.rs`, `examples/manage_nest_agent_sessions.rs`, run against `api.anthropic.com` with a real key from `.env`, gitignored):
  - Skills: `create_skill` created a real `nest-core` skill (uncovered the `files[]` field-name bug above), `list_skills` correctly parsed both `custom`- and `anthropic`-source skills in one response, `get_skill` round-tripped the created skill.
  - Agents: `create_agent` created a real agent with the prebuilt `agent_toolset` tool and the `nest-core` skill attached (found by name via `list_skills`, not hardcoded); `list_agents`/`get_agent` round-tripped it, confirming `AgentModelInfo`, `AgentTool::AgentToolset`, and `AgentSkillRef::Custom` (including the server-side `version: "latest"` default) all deserialize correctly from real responses.
  - `update_agent`/`archive_agent`/`delete_skill` remain wiremock-only — no reason to mutate or permanently archive the real resources created above just to exercise those two calls.
  - Sessions: all six operations (`create_session`/`get_session`/`list_sessions`/`update_session`/`archive_session`/`delete_session`) round-tripped against a real session on the "Nest Framework Agent", confirming archive-then-delete on the same session composes without conflict (unlike Agents, which have no delete). No messages were sent — the session stayed `idle` and no model inference was billed.

## Live resources

Created on the account whose key lives in the repo's gitignored `.env` — **don't re-run the create examples**, they'd create duplicates; use `update_agent` to attach more skills to the existing agent as they're built, and reuse the environment below for any new session examples.

| Resource | id | Notes |
|---|---|---|
| Skill | `skill_01PyeSaH6YjXWv2kw7BrAaRA` | display_title `nest-core`, from `examples/nest-core-skill/SKILL.md` |
| Agent | `agent_01DwppBhxh6j4aKQmFbPhJ5H` | "Nest Framework Agent", `claude-opus-4-8`, `agent_toolset_20260401`, skills: [nest-core] |
| Environment | `env_01HL2gDPYGCxBDnQEsq7N91N` | name `nest-desktop`, `cloud`/`unrestricted` — created by `examples/manage_nest_agent_sessions.rs`, found-by-name (not hardcoded) so it's safe to re-run |

## Related

- [nest-claude README](../nest-claude/README.md)
- [nest-tmdb v1](nest-tmdb-v1.md) — the provider-adapter pattern this crate follows
- [nest-http-client README](../nest-http-client/README.md)
- [Claude API docs](https://platform.claude.com/docs/en/api/overview)
