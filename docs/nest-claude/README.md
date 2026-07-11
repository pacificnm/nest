# nest-claude

Claude (Anthropic) Messages API client for the [Nest framework](../../README.md).

**Crate path:** [`modules/crates/nest-claude`](../../modules/crates/nest-claude)

## Role

`nest-claude` wraps `POST /v1/messages` — non-streaming and streaming (SSE) requests, text/image content, tool use, extended (adaptive) thinking + effort, and prompt caching — behind [`ClaudeClient`]. It does not implement an agent loop, MCP, or the Files/Batches APIs.

## Quick start

```rust
use nest_claude::prelude::*;

#[tokio::main]
async fn main() -> ClaudeResult<()> {
    let client = ClaudeClient::new(ClaudeConfig::from_env()?)?;

    let response = client
        .create_message(CreateMessageRequest::new(vec![Message::user(
            "What is the capital of France?",
        )]))
        .await?;

    println!("{}", response.text());
    Ok(())
}
```

Or as a Nest module:

```rust
use nest_claude::{ClaudeConfig, ClaudeModule};
use nest_core::AppBuilder;

let built = AppBuilder::new()
    .module(ClaudeModule::with_config(
        ClaudeConfig::builder().api_key("sk-ant-...").build()?,
    ))
    .build()?;

let client = built.context.service::<nest_claude::ClaudeClient>()?;
```

## Configuration

```toml
[claude]
# api_key_env = "ANTHROPIC_API_KEY"  # default; or set api_key inline
default_model = "claude-opus-4-8"
default_max_tokens = 4096
```

Export the key before running:

```bash
export ANTHROPIC_API_KEY="sk-ant-..."
```

`default_model`/`default_max_tokens` are used whenever a request doesn't set `.model(...)` / `.max_tokens(...)` explicitly.

## Streaming

```rust
use futures_util::StreamExt;
use nest_claude::prelude::*;

let mut stream = client
    .stream_message(CreateMessageRequest::new(vec![Message::user("Write a haiku")]))
    .await?;

while let Some(event) = stream.next().await {
    if let StreamEvent::ContentBlockDelta {
        delta: ContentDelta::TextDelta { text },
        ..
    } = event?
    {
        print!("{text}");
    }
}
```

## Tool use

```rust
let request = CreateMessageRequest::new(vec![Message::user("What's the weather in Paris?")])
    .tools(vec![ToolDefinition::new(
        "get_weather",
        "Get the current weather for a location",
        serde_json::json!({
            "type": "object",
            "properties": {"location": {"type": "string"}},
            "required": ["location"]
        }),
    )]);

let response = client.create_message(request).await?;
for (tool_use_id, name, input) in response.tool_uses() {
    // execute the tool, then send a `ContentBlock::tool_result(tool_use_id, output)`
    // back as a user turn alongside the echoed assistant `response.content`.
}
```

## Extended thinking and prompt caching

```rust
let request = CreateMessageRequest::new(messages)
    .thinking(ThinkingConfig::adaptive())
    .output_config(OutputConfig { effort: Some(Effort::High) })
    .system(SystemPrompt::cached(large_system_prompt, CacheControl::ephemeral()));
```

## Counting tokens

`CountTokensRequest` mirrors `CreateMessageRequest` (messages, system, tools, tool_choice, thinking, output_config, cache_control) minus `max_tokens`/`stream` — there's no response to cap or stream, only input tokens to count. Token counts are model-specific; pass the same model you'll use for the real request.

```rust
let count = client
    .count_tokens(CountTokensRequest::new(vec![Message::user("Hello, world")]).model("claude-opus-4-8"))
    .await?;

println!("{} input tokens", count.input_tokens);
```

## Skills

Every Skills endpoint requires the `anthropic-beta: skills-2025-10-02` header — `nest-claude` sends it per-request (not as a default header) since it doesn't apply to `/v1/messages`. `create_skill` uploads files as `multipart/form-data`; the API derives `display_title`/`directory`/`description` from the uploaded `SKILL.md` rather than from separate request fields, so give every file a common top-level directory prefix (e.g. `"my-skill/SKILL.md"`, `"my-skill/scripts/run.py"`).

```rust
let skill = client
    .create_skill(vec![FileUpload::new(
        "my-skill/SKILL.md",
        std::fs::read("my-skill/SKILL.md")?,
    )])
    .await?;

let page = client
    .list_skills(ListSkillsParams::new().limit(50).source(SkillSource::Custom))
    .await?;

let same_skill = client.get_skill(&skill.id).await?;
client.delete_skill(&skill.id).await?;
```

Skill *versions* (`POST/GET/DELETE /v1/skills/{id}/versions[/{version}]`, content download) are not yet covered — each `create_skill` call creates the skill's first version implicitly, but there's no way yet to add a new version to an existing skill.

> **Field name is `files[]`, not `files`.** The API's own example (`-F files=...`) is misleading — a bare `files` field returns `400 files[]: Field required`. `encode_multipart_files` sends `name="files[]"` on each part; confirmed against the live API (see `examples/create_nest_core_skill.rs`).

Live-verified end to end: `examples/create_nest_core_skill.rs` creates a real "nest-core" skill from `examples/nest-core-skill/SKILL.md` (documenting the `nest-core` crate for Claude), and `examples/list_skills.rs` lists skills and reads one back. Run with `ANTHROPIC_API_KEY` set:

```bash
cargo run --example create_nest_core_skill -p nest-claude
cargo run --example list_skills -p nest-claude
```

## Agents (Managed Agents)

Every Agents endpoint requires `anthropic-beta: managed-agents-2026-04-01`, sent per-request like the Skills beta header. An agent is a persisted, versioned config — `model`/`system`/`tools`/`skills`/`mcp_servers` live on the agent, never on a session (sessions aren't covered yet, so an agent alone can't run — this crate only manages the resource, not execution).

```rust
let agent = client
    .create_agent(
        CreateAgentRequest::new("My First Agent", "claude-sonnet-4-6")
            .system("You are a helpful coding assistant.")
            .tools(vec![AgentTool::agent_toolset()]),
    )
    .await?;

let page = client
    .list_agents(ListAgentsParams::new().include_archived(false))
    .await?;

let same_agent = client.get_agent(&agent.id, None).await?; // omit `version` for latest

// Update creates a new version; pin `version` to the agent's current one
// (optimistic concurrency lock) — fields you omit are preserved.
let updated = client
    .update_agent(
        &agent.id,
        UpdateAgentRequest::new(agent.version).system("Updated system prompt."),
    )
    .await?;
assert_eq!(updated.version, agent.version + 1);

// Archiving is permanent — read-only afterward, no unarchive.
client.archive_agent(&agent.id).await?;
```

`model` accepts a bare string (`AgentModel::id(...)`, or just `.into()` from `&str`/`String`) or `{id, speed}` (`AgentModel::with_speed(...)`) — responses always echo the object form as [`AgentModelInfo`]. `UpdateAgentRequest` has `clear_system()`/`clear_tools()`/`clear_skills()`/`clear_mcp_servers()` helpers (the API clears a field on an empty string/array, not by omitting it) and a `metadata` patch mode (`Some(value)` upserts a key, `None` deletes it).

Not yet covered: environments, vaults, deployments, Session Events, multi-agent execution (the `multiagent`/`AgentRef` roster type round-trips on the agent object, but Session Events aren't covered yet, so there's no way to actually drive a coordinator), and agent versions listing (`GET /v1/agents/{id}/versions`).

Live-verified: `examples/create_nest_agent.rs` looks up the "nest-core" skill by name (via `list_skills`) and creates a real "Nest Framework Agent" with the prebuilt `agent_toolset` and that skill attached; `examples/list_agents.rs` lists agents and reads one back (`update_agent`/`archive_agent` remain wiremock-only — no need to mutate or permanently archive the real agent just to test them). See `docs/plan/nest-claude-v1.md` § Live resources for the actual ids this produced — **don't re-run `create_nest_core_skill`/`create_nest_agent`**, they'd create duplicates; use `update_agent` to attach further skills to the existing agent instead.

```bash
cargo run --example create_nest_agent -p nest-claude
cargo run --example list_agents -p nest-claude
```

## Sessions (Managed Agents)

A session is a running (or terminated) instance of a pre-created agent — it's meaningless without one, and it also requires an `environment_id` (the Environments API isn't wrapped by this crate yet; see below). Every Sessions endpoint requires the same `anthropic-beta: managed-agents-2026-04-01` header as Agents.

```rust
let session = client
    .create_session(
        CreateSessionRequest::new(SessionAgentRef::id(agent.id), environment_id)
            .title("My session"),
    )
    .await?;

let same_session = client.get_session(&session.id).await?;

let page = client.list_sessions(ListSessionsParams::new().limit(20)).await?;

let renamed = client
    .update_session(&session.id, UpdateSessionRequest::new().title("Renamed"))
    .await?;

// Archive is permanent (read-only afterward, no unarchive); delete removes
// the session, its event history, container, and checkpoints entirely — the
// two compose fine back-to-back on the same session.
client.archive_session(&session.id).await?;
client.delete_session(&session.id).await?;
```

`SessionAgentRef` accepts a bare agent-id string (`.into()` from `&str`/`String`, latest version) or `SessionAgentRef::versioned(id, version)` — the third wire shape, `agent_with_overrides` (session-local override of model/system/tools/mcp_servers/skills), isn't covered yet. `UpdateSessionRequest` supports `title`/`metadata` only — the mid-session `agent.tools`/`agent.mcp_servers`/`vault_ids` override (valid only while `idle`) isn't covered either. `SessionListPage` is the one Managed Agents list page with backward pagination (`prev_page`, alongside `next_page`).

Not yet covered: Session Events (sending messages, streaming replies — a created session can't yet hold a live conversation), environments, vaults, session resources, and session versions/threads.

Live-verified: `examples/manage_nest_agent_sessions.rs` exercises all six operations against a real session on the "Nest Framework Agent," including archiving then deleting the same session back-to-back. It also finds-or-creates one reusable `env_...` (named `nest-desktop`) via two raw `nest_http_client` calls, since there's no `environments.rs` yet — see `docs/plan/nest-claude-v1.md` § Live resources for the id this produced. No messages are sent, so the session never leaves `idle` and no model inference is billed.

```bash
cargo run --example manage_nest_agent_sessions -p nest-claude
```

## Client API

| Method | Endpoint | Description |
|--------|----------|-------------|
| `create_message` | `POST /v1/messages` | Non-streaming, returns [`MessageResponse`] |
| `stream_message` | `POST /v1/messages` (`stream: true`) | Returns a [`MessageStream`] of [`StreamEvent`] |
| `count_tokens` | `POST /v1/messages/count_tokens` | Returns [`TokenCountResponse`] without creating a message |
| `create_skill` | `POST /v1/skills` | Multipart file upload, returns [`Skill`] |
| `list_skills` | `GET /v1/skills` | Returns a [`SkillListPage`] (`limit`/`page`/`source` filters) |
| `get_skill` | `GET /v1/skills/{skill_id}` | Returns [`Skill`] |
| `delete_skill` | `DELETE /v1/skills/{skill_id}` | Returns [`SkillDeleted`] |
| `create_agent` | `POST /v1/agents` | Returns [`Agent`] |
| `list_agents` | `GET /v1/agents` | Returns an [`AgentListPage`] (`limit`/`page`/`include_archived`/`created_at_gte`/`created_at_lte` filters) |
| `get_agent` | `GET /v1/agents/{agent_id}` | Returns [`Agent`], optionally pinned to a `version` |
| `update_agent` | `POST /v1/agents/{agent_id}` | Version-locked patch, returns the new [`Agent`] version |
| `archive_agent` | `POST /v1/agents/{agent_id}/archive` | Permanent; returns [`Agent`] with `archived_at` set |
| `create_session` | `POST /v1/sessions` | Returns [`Session`] |
| `list_sessions` | `GET /v1/sessions` | Returns a [`SessionListPage`] (`limit`/`page`/`order` filters; the one list page with `prev_page`) |
| `get_session` | `GET /v1/sessions/{session_id}` | Returns [`Session`] |
| `update_session` | `POST /v1/sessions/{session_id}` | Patches `title`/`metadata`, returns the updated [`Session`] |
| `delete_session` | `DELETE /v1/sessions/{session_id}` | Permanent; returns [`SessionDeleted`] |
| `archive_session` | `POST /v1/sessions/{session_id}/archive` | Permanent (read-only); returns [`Session`] with `archived_at` set |

## Logging and errors

- Emits `tracing` events only — no `nest-logging` dependency, per [`nest-logging` overview](../nest-logging/overview.md); host apps install the subscriber.
- `ClaudeError` maps Anthropic's HTTP status codes to kinds (`InvalidRequest` 400, `Auth` 401/403, `RateLimit` 429, `Server` 5xx/529) and converts to [`NestError`] with `NestErrorKind::Network` or `NestErrorKind::Validation`.

## Endpoints covered

- `POST /v1/messages` — non-streaming and streaming (SSE), text/image content, tool use, adaptive thinking + effort, prompt caching (`cache_control`)
- `POST /v1/messages/count_tokens`
- Skills: Create, List, Get, Delete (`/v1/skills`, `/v1/skills/{skill_id}`)
- Agents: Create, List, Get, Update, Archive (`/v1/agents`, `/v1/agents/{agent_id}`, `/v1/agents/{agent_id}/archive`)
- Sessions: Create, List, Get, Update, Delete, Archive (`/v1/sessions`, `/v1/sessions/{session_id}`, `/v1/sessions/{session_id}/archive`)

Not yet covered: Files API, PDF/document content blocks, server-side tools (web search, code execution), the Batches API, the Models API, skill versions, agent/session versions, Session Events, and Managed Agents environments/vaults, structured outputs (`output_config.format`).

## Related

- [nest-http-client](../nest-http-client/README.md) — HTTP transport (`nest-claude` owns a dedicated instance, not the shared one, so the API key never leaks into other modules)
- [nest-error](../nest-error/README.md) — `NestError`/`NestResult`
- [Claude API docs](https://platform.claude.com/docs/en/api/overview)
