# Phase 3 Task Spec — Build `nest-ai-claude`

**Repo:** `pacificnm/nest` (framework repo)
**New crate:** `modules/crates/nest-ai-claude`
**Branch:** `feature/nest-ai-claude-v1`

## Ground truth (verified against the real repo — read before starting)

- **`nest-ai`'s trait** (`core/crates/nest-ai/src/provider.rs`):
  ```rust
  #[async_trait]
  pub trait AiProvider: Send + Sync {
      fn provider_id(&self) -> &'static str;
      async fn complete(&self, request: CompletionRequest) -> AiResult<CompletionResponse>;
      async fn stream_complete(&self, _request: CompletionRequest) -> AiResult<CompletionStream> {
          Err(AiError::invalid_input("streaming is not supported by this provider")) // default
      }
  }
  ```
- **`nest-ai`'s types** (`core/crates/nest-ai/src/types.rs`, `tools.rs`):
  - `CompletionRequest { model: Option<String>, messages: Vec<ChatMessage>, format: Option<ResponseFormat>, tools: Vec<ToolDefinition> }`
  - `ChatMessage { role: ChatRole, content: String, tool_name: Option<String>, tool_calls: Option<Vec<ToolCall>> }`, `ChatRole::{System, User, Assistant, Tool}`
  - `ToolDefinition { name, description, parameters: Value }`, `ToolCall { id: String, name: String, arguments: Value }`
  - `CompletionResponse { model, content, done, tool_calls: Vec<ToolCall>, metrics: Option<CompletionMetrics> }`
  - `CompletionChunk { content_delta, done, metrics, tool_calls }`, `CompletionStream = Pin<Box<dyn Stream<Item = AiResult<CompletionChunk>> + Send>>`
  - `merge_tool_calls(&mut Vec<ToolCall>, &[ToolCall])` — exported helper for streaming accumulation, reuse it, do not reimplement.
- **`nest-claude`'s client** (`modules/crates/nest-claude/src/client.rs`): `ClaudeClient::new(ClaudeConfig) -> ClaudeResult<Self>`, `create_message(CreateMessageRequest) -> ClaudeResult<MessageResponse>`, `stream_message(...) -> ClaudeResult<MessageStream>`. `ClaudeClient` owns its **own dedicated** `HttpClientService` internally (not the process-wide shared one) — mirror this ownership model, do not try to share a client across modules.
- **`nest-claude`'s request types** (`request.rs`): `Message { role: Role, content: Vec<ContentBlock> }` with constructors `Message::user(text)`, `Message::assistant(text)`, `Message::user_blocks(...)`; `SystemPrompt::Text(String)` (separate top-level field on the request, **not** a message in the list); `ToolDefinition { name, description, input_schema, cache_control }` with `ToolDefinition::new(name, description, input_schema)`.
- **`nest-claude`'s response types** (`response.rs`): `MessageResponse::text() -> String` (concatenates text blocks), `MessageResponse::tool_uses() -> impl Iterator<Item = (&str, &str, &Value)>` yielding `(id, name, input)`.
- **`nest-claude`'s `ContentBlock::tool_result`** (`types.rs`): `ContentBlock::tool_result(tool_use_id: impl Into<String>, ...)` — **requires** the id of the `tool_use` block it answers. This is the crux of the whole adapter; see §Design below.
- **`nest-claude`'s config** (`config.rs`): `ClaudeConfig::builder().api_key(...).base_url(...).build()` — `base_url` is overridable, confirmed, so tests can point at a `wiremock::MockServer` exactly like `nest-ai-ollama`'s tests point at one for `/api/chat`.
- **`nest-claude`'s streaming** (`stream.rs`): `StreamEvent::ContentBlockDelta { index, delta: ContentDelta }`, `ContentDelta::{TextDelta{text}, InputJsonDelta{partial_json}, ThinkingDelta{..}, SignatureDelta{..}}`. Tool-call streaming arrives as raw partial-JSON fragments per content-block `index`, structurally different from Ollama's per-object incremental `tool_calls` — **do not** attempt to reassemble streaming tool calls in this phase; see scope note below.
- **Pattern to mirror throughout:** `modules/crates/nest-ai-ollama` (`config.rs` — skip, reuse `ClaudeConfig` directly; `provider.rs`, `module.rs`, its own `error.rs`).

---

## Design — the tool-result correlation problem

`nest_ai::ChatMessage::tool_result(name, content)` carries a tool **name**,
never an id. Claude's API requires the `tool_use_id` of the specific call a
result answers, not just a name. `nest_ai::ToolCall` (used in
`ChatMessage.tool_calls` on assistant turns) **does** carry an `id` — so the
data exists in the conversation history, just not on the `Tool`-role message
itself.

**Resolution (do this, do not invent a different approach without flagging
it):** when converting `request.messages: Vec<ChatMessage>` into Claude
`Message`s, walk the list in order and maintain a `HashMap<String, String>`
(tool name → most recent `tool_use_id`), populated every time you encounter
an `Assistant` message with `tool_calls: Some(calls)` (from `calls`, keyed by
`call.name`, valued by `call.id`). When you subsequently encounter a `Tool`
role message, look up `tool_name` in that map to get the `tool_use_id` to
pass into `ContentBlock::tool_result(id, ...)`.

**Known limitation, document it in the crate's doc comments, do not silently
paper over it:** if the model calls the same tool name twice in a single
turn (parallel calls with duplicate names), this lookup only keeps the last
one. Acceptable for v1 — Sparrow's own tool set (Phase 10) doesn't call the
same tool twice per turn — but a future caller doing that will get wrong
correlation silently unless this is documented clearly on the public
conversion function.

---

## Crate layout

```
modules/crates/nest-ai-claude/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── message.rs   # ChatMessage <-> nest_claude::Message conversions (the hard part)
    ├── provider.rs   # ClaudeAiProvider: AiProvider impl
    ├── module.rs      # ClaudeAiModule: registers AiService(Arc::new(ClaudeAiProvider))
    └── error.rs       # ClaudeError -> AiError mapping
```

`Cargo.toml` dependencies: `nest-ai = { workspace = true }`, `nest-claude = { workspace = true }`, `nest-core = { workspace = true }`, `nest-error = { workspace = true }`, `async-trait`, `futures-util`, `serde_json`. Dev-dependency: `wiremock` (check the exact version `nest-ai-ollama`'s own `Cargo.toml` pins and match it — do not pin a different one for no reason).

### `message.rs`

```rust
use std::collections::HashMap;
use nest_ai::{ChatMessage, ChatRole};
use nest_claude::request::{Message, SystemPrompt};
use nest_claude::types::ContentBlock;

/// Converts nest-ai chat history into a Claude system prompt (if any System
/// messages are present) plus the remaining turns as Claude `Message`s.
///
/// See the crate-level docs for the tool_use_id correlation limitation this
/// function relies on (last-call-wins per tool name, per conversation).
pub fn to_claude_messages(messages: &[ChatMessage]) -> (Option<SystemPrompt>, Vec<Message>) {
    let mut system_parts = Vec::new();
    let mut claude_messages = Vec::new();
    let mut last_tool_use_id_by_name: HashMap<String, String> = HashMap::new();

    for msg in messages {
        match msg.role {
            ChatRole::System => system_parts.push(msg.content.clone()),
            ChatRole::User => claude_messages.push(Message::user(&msg.content)),
            ChatRole::Assistant => {
                if let Some(calls) = &msg.tool_calls {
                    for call in calls {
                        last_tool_use_id_by_name.insert(call.name.clone(), call.id.clone());
                    }
                    // ASSISTANT TURN WITH TOOL CALLS: build tool_use content blocks too,
                    // not just text — check ContentBlock's tool_use constructor name in
                    // types.rs before writing this (do not guess the method name).
                    todo!("build Message::assistant_blocks(...) including tool_use blocks per call")
                } else {
                    claude_messages.push(Message::assistant(&msg.content));
                }
            }
            ChatRole::Tool => {
                let tool_use_id = msg
                    .tool_name
                    .as_deref()
                    .and_then(|name| last_tool_use_id_by_name.get(name))
                    .cloned()
                    .unwrap_or_default(); // TODO: decide whether an empty id should be a hard
                                          // error instead — check what the live API does with
                                          // an empty tool_use_id before shipping silently.
                claude_messages.push(Message::user_blocks(vec![ContentBlock::tool_result(
                    tool_use_id,
                    &msg.content, // check tool_result's exact signature — it may take
                                  // structured content, not just a string; verify in types.rs.
                )]));
            }
        }
    }

    let system = if system_parts.is_empty() {
        None
    } else {
        Some(SystemPrompt::text(system_parts.join("\n\n")))
    };

    (system, claude_messages)
}

/// Converts nest-ai `ToolDefinition`s into Claude's.
pub fn to_claude_tools(tools: &[nest_ai::ToolDefinition]) -> Vec<nest_claude::request::ToolDefinition> {
    tools
        .iter()
        .map(|t| nest_claude::request::ToolDefinition::new(&t.name, &t.description, t.parameters.clone()))
        .collect()
}
```

**Explicit unresolved items — verify against real `types.rs`/`request.rs` before writing, do not guess:**
- The exact constructor for a `tool_use` content block on the assistant side (needed to echo the model's own tool call back into history on the next turn — Claude requires the full prior assistant turn including its `tool_use` blocks, not just the tool result).
- Whether `ContentBlock::tool_result`'s second parameter is a plain string or a structured content type.

### `provider.rs`

```rust
use async_trait::async_trait;
use nest_ai::{AiProvider, AiResult, CompletionRequest, CompletionResponse, CompletionStream, ToolCall};
use nest_claude::{ClaudeClient, ClaudeConfig};
use nest_claude::request::CreateMessageRequest;

#[derive(Clone)]
pub struct ClaudeAiProvider {
    client: ClaudeClient,
}

impl ClaudeAiProvider {
    pub fn new(config: ClaudeConfig) -> nest_error::NestResult<Self> {
        Ok(Self { client: ClaudeClient::new(config).map_err(nest_error::NestError::from)? })
    }
}

#[async_trait]
impl AiProvider for ClaudeAiProvider {
    fn provider_id(&self) -> &'static str {
        "claude"
    }

    async fn complete(&self, request: CompletionRequest) -> AiResult<CompletionResponse> {
        if request.messages.is_empty() {
            return Err(nest_ai::AiError::invalid_input(
                "completion request requires at least one message",
            ));
        }

        let (system, messages) = crate::message::to_claude_messages(&request.messages);
        let mut claude_request = CreateMessageRequest::new(messages);
        if let Some(system) = system {
            claude_request = claude_request.system(system);
        }
        if let Some(model) = &request.model {
            claude_request = claude_request.model(model);
        }
        if !request.tools.is_empty() {
            claude_request = claude_request.tools(crate::message::to_claude_tools(&request.tools));
        }
        // CHECK: does CreateMessageRequest have a builder method for the response-format
        // hint (request.format == Json)? If not, note that as a known gap — Claude's API
        // does not have a direct "JSON mode" equivalent the way Ollama does; the honest
        // answer may be "unsupported for this provider," not a silent no-op.

        let response = self
            .client
            .create_message(claude_request)
            .await
            .map_err(crate::error::claude_to_ai_error)?;

        let tool_calls: Vec<ToolCall> = response
            .tool_uses()
            .map(|(id, name, input)| ToolCall {
                id: id.to_string(),
                name: name.to_string(),
                arguments: input.clone(),
            })
            .collect();

        Ok(CompletionResponse {
            model: response.model.clone(),
            content: response.text(),
            done: true,
            tool_calls,
            metrics: None, // CHECK: does response.usage map sensibly onto
                           // nest_ai::metrics::CompletionMetrics's fields? If the shapes
                           // are compatible, map it; if not, leaving this None is fine
                           // for v1 but note the gap in a doc comment, don't just drop it silently.
        })
    }

    async fn stream_complete(&self, request: CompletionRequest) -> AiResult<CompletionStream> {
        // SCOPE LIMIT FOR THIS PHASE: only support streaming for tool-free requests.
        // Reassembling Claude's per-content-block partial-JSON tool deltas into
        // nest_ai's ToolCall shape is real, non-trivial work (structurally different
        // from Ollama's per-object incremental tool_calls) — do not attempt it in this
        // phase. Fall back to the trait's default "not supported" behavior when tools
        // are present instead of shipping a half-working implementation.
        if !request.tools.is_empty() {
            return Err(nest_ai::AiError::invalid_input(
                "streaming with tools is not yet supported by nest-ai-claude",
            ));
        }
        todo!("map ClaudeClient::stream_message's StreamEvent::ContentBlockDelta{{delta: ContentDelta::TextDelta{{text}}}} into CompletionChunk::delta(text); MessageDelta/MessageStop into CompletionChunk::finished()")
    }
}
```

### `error.rs`

Map `ClaudeErrorKind` (from `nest-claude`'s `error.rs`: `Config, Http, InvalidRequest, Auth, RateLimit, Server, Parse, Api`) onto `nest_ai::AiErrorKind` (`InvalidInput, Request, Parse, Config`):

```rust
pub fn claude_to_ai_error(error: nest_claude::ClaudeError) -> nest_ai::AiError {
    use nest_claude::ClaudeErrorKind::*;
    let kind = match error.kind() {
        Config => nest_ai::AiErrorKind::Config,
        InvalidRequest => nest_ai::AiErrorKind::InvalidInput,
        Parse => nest_ai::AiErrorKind::Parse,
        Http | Auth | RateLimit | Server | Api => nest_ai::AiErrorKind::Request,
    };
    nest_ai::AiError::new(kind, error.to_string())
}
```

Confirm `ClaudeError` actually exposes a `.kind()` accessor with this exact name before writing this (it does, per `error.rs`'s `pub fn kind(&self) -> ClaudeErrorKind` pattern seen elsewhere in the module — but double check on the real file, don't copy blind).

### `module.rs`

Mirror `OllamaModule` exactly:

```rust
pub const CLAUDE_AI_MODULE_ID: ModuleId = ModuleId("nest-ai-claude");

pub struct ClaudeAiModule {
    config: Option<ClaudeConfig>,
}

impl ClaudeAiModule {
    pub fn new() -> Self { Self { config: None } }
    pub fn with_config(config: ClaudeConfig) -> Self { Self { config: Some(config) } }
}

impl Module for ClaudeAiModule {
    fn id(&self) -> ModuleId { CLAUDE_AI_MODULE_ID }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        let config = match &self.config {
            Some(config) => config.clone(),
            None => {
                let config_service = app.service_mut::<nest_config::ConfigService>()?;
                ClaudeConfig::from_config_service(config_service)?
            }
        };
        let provider = ClaudeAiProvider::new(config)?;
        app.register_service(AiService::new(std::sync::Arc::new(provider)))
    }
}
```

Note: `ClaudeConfig::from_config_service` reads the `[claude]` TOML section
(same one `nest-claude`'s own `ClaudeModule` reads) — this means
`ClaudeAiModule` and `ClaudeModule` would read the **same config section** if
both are registered in the same app. That's fine and intentional (one API
key, one set of defaults) but worth a doc comment saying so explicitly, since
it's not obvious from the module name alone.

---

## Tests

Mirror `nest-ai-ollama/src/provider.rs`'s test style exactly — `wiremock::MockServer`, mounted on `POST /v1/messages`, `ClaudeConfig::builder().api_key("test-key").base_url(server.uri()).build()`:

1. `complete_uses_messages_endpoint` — mock a plain text response, assert `CompletionResponse.content` matches.
2. `complete_rejects_empty_messages` — assert `AiErrorKind::InvalidInput`.
3. `complete_with_tools_returns_tool_calls` — mock a `tool_use` content block response, assert `tool_calls[0].id`/`.name`/`.arguments` round-trip correctly (this is the test that actually proves the id-carrying path works, not just the name).
4. `tool_result_uses_correct_tool_use_id` — the most important new test in this crate: build a `CompletionRequest` with an assistant turn containing two *different-named* tool calls, followed by two `Tool`-role result messages; assert (by inspecting the outgoing HTTP request body via `wiremock`'s request capture, not just the response) that each `tool_result` block carries the correct, distinct `tool_use_id` — this is the test that would catch a broken correlation map.
5. `stream_complete_emits_text_chunks` — tool-free streaming case.
6. `stream_complete_with_tools_returns_unsupported_error` — confirms the explicit scope limit is enforced, not silently ignored.

**Acceptance for Phase 3:** `cargo test -p nest-ai-claude` passes (no live API key needed — everything mocked via `wiremock`, matching `nest-claude`'s own test conventions); `cargo doc -p nest-ai-claude` builds cleanly; a standalone example runs the **same** `CompletionRequest` against both `nest-ai-ollama` and `nest-ai-claude` (with a real or mocked backend for each) and gets a valid `CompletionResponse` from both — this is the actual proof that the `AiProvider` swap works, not just that each crate compiles in isolation.

## Explicit "do not" list

- Do not guess the `tool_use` content-block constructor name or `ContentBlock::tool_result`'s exact parameter types — both are flagged `todo!()`/comments above specifically because they need verification against the real `types.rs`, not because they're unimportant.
- Do not attempt streaming tool-call reassembly in this phase — return the explicit "not yet supported" error instead of a partial/buggy implementation.
- Do not let `ClaudeAiProvider` reuse a shared `HttpClientService` — `ClaudeClient` already owns its own dedicated one specifically so the API key doesn't leak into other modules; preserve that.
- Do not silently drop the tool-name-collision limitation in `to_claude_messages` — it must be documented on the function itself, not just in this spec.
