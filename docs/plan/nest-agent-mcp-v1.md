# nest-agent + nest-mcp v1 Implementation Plan

## Status: Complete (v1)

Kiwi copy: [`apps/kiwi/docs/agent-mcp-v1.md`](../../apps/kiwi/docs/agent-mcp-v1.md).

Enable Kiwi (and other Nest hosts) to run a **tool-using agent loop** against local
Ollama models, with tools supplied by **MCP servers** — starting with the existing
Nest memory servers used by Cursor today.

## Context

### Today

| Component | State |
| --- | --- |
| Kiwi chat | Single-turn streaming chat via `nest-ai` → `nest-ai-ollama` → Ollama `/api/chat` |
| Nest MCP servers | Python FastMCP stdio servers (`tools/mcp_*.py`) wired in `.cursor/mcp.json` |
| MCP client in Rust | None |
| Tool / agent types in `nest-ai` | None |
| Ollama tool calling | Not implemented in `nest-ai-ollama` |

Cursor hosts MCP and runs an agent loop externally. Kiwi must become an MCP **client**
and run its own **agent loop** inside the Nest module graph.

### Goal

A user asks Kiwi a question; the model can call MCP tools (e.g.
`search_project_memory`), receive results, and produce a final answer — with tool
steps visible in the chat UI.

### Non-goals (v1)

- Replacing Cursor's MCP host or hooks
- HTTP/SSE MCP transport (stdio only)
- MCP **server** implementation in Rust (reuse existing Python servers)
- Write/edit file tools, shell execution, or arbitrary code execution
- Tool approval UI beyond a global read-only auto-run policy
- Cloud providers (OpenAI tools API) — Ollama first
- Persistent agent run history / replay

---

## Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│ Kiwi (apps/kiwi)                                            │
│  workbench chat UI  ←→  AgentRunner  ←→  AgentService       │
└───────────────────────────────┬─────────────────────────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        ▼                       ▼                       ▼
  nest-agent              nest-ai                 nest-mcp
  (loop, events)     (tools, messages)      (stdio JSON-RPC client)
        │                       │                       │
        └───────────────────────┼───────────────────────┘
                                ▼
                        nest-ai-ollama
                     (tools field, tool_calls parse)
                                │
                                ▼
                          Ollama /api/chat
                                │
                                ▼
                    MCP server child processes
              (nest-memory, nest-knowledge, nest-context-memory)
```

### Agent loop (multi-turn)

```text
1. Build messages + tool schemas from McpToolRegistry
2. Call LLM (stream or non-stream)
3. If assistant message contains tool_calls:
     a. For each call → McpClient.tools/call
     b. Append tool result messages
     c. Goto 2 (until no tool_calls or max_steps)
4. Emit final assistant text + metrics
```

Reference: [Ollama tool calling](https://docs.ollama.com/capabilities/tool-calling).

---

## Crate boundaries

Respect [architecture.md](../architecture.md): **core** defines contracts; **modules**
implement integrations; **apps** wire UX.

| Crate | Layer | Role |
| --- | --- | --- |
| `nest-mcp` | **core** | MCP protocol types, stdio transport, session lifecycle (no Tokio required in types crate — see split note below) |
| `nest-mcp-runtime` | **core** | Async process spawn, read/write loops, `McpClient`, `McpHub` (Tokio) — *or* fold into `nest-mcp` if small |
| `nest-agent` | **core** | `AgentLoop`, `ToolRegistry`, events, step limits, cancellation (host-agnostic) |
| `nest-ai` | **core** | Extend with tool-aware messages, `ToolDefinition`, `CompletionRequest.tools`, provider trait |
| `nest-ai-ollama` | **module** | Ollama `tools` request field, parse `tool_calls` from stream + final chunk |
| `nest-mcp-module` | **module** | `McpModule`, load config, register `McpHubService` — *optional v1.1; Kiwi can construct hub directly* |
| Kiwi | **app** | Config, Agent sidebar MCP status, chat UI for tool steps, CLI `kiwi agent` |

**Dependency rule:** `nest-mcp` / `nest-agent` must not depend on modules or apps.
`nest-ai-ollama` depends on `nest-ai` only (not `nest-mcp`). The **app** (or a
future `nest-mcp-module`) connects `McpHub` tools into `AgentLoop`.

**Crate split decision (v1):** Start with a single `core/crates/nest-mcp` crate
(async Tokio inside, like `nest-http-client`). Split later if the protocol layer
needs to stay sync-only.

---

## Phase 0 — Prerequisites and spike

**Objective:** Prove Ollama tool calling works with a target model before building MCP.

| Task | Owner | Done when |
| --- | --- | --- |
| Document model requirements | Plan | Table of tested models (see below) |
| Manual curl / script spike | Dev | One tool round-trip against Ollama with `qwen2.5-coder:3b` or recommended model |
| Confirm Kiwi config path to repo root | Dev | Agent can resolve `.cursor/mcp.json` and `.env` when cwd is `apps/kiwi/desktop` |

**Model guidance**

| Model | Tool calling (expected) | Notes |
| --- | --- | --- |
| `qwen2.5-coder:3b` | Weak / unreliable | Current Kiwi default — OK for chat, not for agent v1 QA |
| `qwen2.5:7b` or `llama3.1:8b` | Good | Recommended minimum for agent MVP |
| `qwen2.5-coder:7b` | Good | Better fit for IDE agent |

Agent v1 tests should run against at least one **7B+ tool-capable** model even if
Kiwi default stays on 3B for chat.

---

## Phase 1 — `nest-mcp` (stdio client)

**Objective:** Spawn one MCP server, list tools, call one tool.

### 1.1 Protocol subset

Implement MCP 2024-11-05 JSON-RPC over stdio:

| Method | Required v1 |
| --- | --- |
| `initialize` | Yes |
| `notifications/initialized` | Yes |
| `tools/list` | Yes |
| `tools/call` | Yes |
| `resources/list` | No (defer) |
| `prompts/list` | No (defer) |

Use `serde_json` for payloads. Map errors to `NestError` via `nest-error` (new
`nest-mcp` module tag, do not invent a parallel error hierarchy).

### 1.2 Types

```rust
// nest-mcp (illustrative)
pub struct McpServerConfig {
    pub name: String,
    pub command: PathBuf,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub env: HashMap<String, String>,
}

pub struct McpTool {
    pub server: String,
    pub name: String,           // fully qualified: "nest-memory/search_project_memory" or namespaced
    pub description: String,
    pub input_schema: JsonValue,
}

pub struct McpHub {
    // manages N child processes
}
impl McpHub {
    pub async fn connect_all(configs: &[McpServerConfig]) -> NestResult<Self>;
    pub async fn list_tools(&self) -> NestResult<Vec<McpTool>>;
    pub async fn call_tool(&self, tool: &str, arguments: JsonValue) -> NestResult<String>;
    pub async fn shutdown(&self) -> NestResult<()>;
}
```

### 1.3 Config loading

- Parse Cursor-compatible JSON:

```json
{
  "mcpServers": {
    "nest-memory": {
      "command": "/path/to/.venv/bin/python",
      "args": ["/path/to/tools/mcp_memory_server.py"],
      "cwd": "/path/to/nest"
    }
  }
}
```

- Loader resolves paths relative to config file directory.
- Kiwi `[agent] mcp_config` in `config.toml` points at repo `.cursor/mcp.json`
  (or embeds inline server list).
- Merge process env with repo `.env` (`DATABASE_URL`, `OPENAI_API_KEY`) for
  memory servers — same as [MCP-SETUP.md](../../tools/MCP-SETUP.md).

### 1.4 Process lifecycle

- Spawn on first agent run (or app startup if configured).
- Stdin/stdout lines framed as **newline-delimited JSON** (verified: FastMCP / MCP Python SDK `stdio.py`).
- Kill child on hub drop / app shutdown.
- Per-call timeout (default 30s; memory search may need 60s for embedding).

### 1.5 Tests

| Test | Type |
| --- | --- |
| Parse `mcp.json` fixture | Unit |
| JSON-RPC serialize round-trip | Unit |
| `tools/list` + `tools/call` against mock stdio server | Integration |
| Live call to `mcp_memory_server.py` (ignored by default, run in dev) | Manual / `#[ignore]` |

**Deliverable:** `cargo test -p nest-mcp` green; manual smoke:

```bash
# future CLI helper or test binary
nest-mcp call nest-memory search_project_memory '{"query":"nest-core","limit":3}'
```

---

## Phase 2 — `nest-ai` tool types + `nest-ai-ollama` tool calling

**Objective:** LLM request/response supports Ollama tools API.

### 2.1 Extend `nest-ai` types

```rust
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: JsonValue, // JSON Schema object
}

pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: JsonValue,
}

pub enum MessageContent {
    Text(String),
    ToolCalls(Vec<ToolCall>),
}

pub struct ChatMessage {
    pub role: ChatRole, // add Tool role or use Assistant + ToolCalls variant
    pub content: MessageContent,
}

pub struct CompletionRequest {
    // existing fields +
    pub tools: Vec<ToolDefinition>,
}
```

**Migration:** Keep backward-compatible helpers (`ChatMessage::user`, etc.) that
produce text-only messages. Existing Kiwi chat path unchanged until Phase 4.

Add to `AiProvider`:

```rust
async fn complete_with_tools(&self, request: CompletionRequest) -> AiResult<CompletionResponse>;
// stream variant accumulates tool_calls from chunks
```

Default impl may delegate to `complete` and ignore tools until provider supports them.

### 2.2 `nest-ai-ollama` changes

| File | Change |
| --- | --- |
| `client.rs` | Add `tools` to `ChatRequestBody`; deserialize `tool_calls` on message |
| `stream.rs` | Accumulate partial `tool_calls` from NDJSON chunks |
| `provider.rs` | Map tool definitions to Ollama `{ type: "function", function: { name, description, parameters } }` |

Ollama tool result message format (verify in spike):

```json
{ "role": "tool", "content": "...", "tool_name": "search_project_memory" }
```

### 2.3 Tests

- Wiremock streaming fixture with `tool_calls` in final chunk
- Non-stream single tool call round-trip
- Provider rejects tools when model returns malformed calls

**Deliverable:** `cargo test -p nest-ai-ollama` with tool fixtures; no MCP yet —
tools can be hard-coded in test.

---

## Phase 3 — `nest-agent` (agent loop)

**Objective:** Orchestrate LLM ↔ tool execution with events for UI/CLI.

### 3.1 Core API

```rust
pub struct AgentConfig {
    pub max_steps: u32,           // default 10
    pub tool_timeout: Duration,
    pub auto_run_policy: AutoRunPolicy, // v1: ReadOnlyOnly
}

pub enum AutoRunPolicy {
    /// Auto-run tools marked read-only (all Nest memory search tools in v1).
    ReadOnlyOnly,
    /// Require explicit approval (v1.1).
    Ask,
}

pub enum AgentEvent {
    TextDelta(String),
    ToolCallStarted { tool: String, arguments: JsonValue },
    ToolCallFinished { tool: String, result: String, duration: Duration },
    ToolCallFailed { tool: String, error: String },
    StepStarted { step: u32 },
    Finished { metrics: Option<CompletionMetrics> },
    Failed(NestError),
}

pub struct AgentLoop {
    ai: AiService,
    tools: ToolRegistry,
}

impl AgentLoop {
    pub async fn run(
        &self,
        messages: Vec<ChatMessage>,
        model: Option<String>,
        tx: mpsc::Sender<AgentEvent>,
        cancel: CancellationToken,
    ) -> NestResult<()>;
}
```

### 3.2 ToolRegistry

- Built from `McpHub::list_tools()` at run start (or cached with TTL).
- Maps Ollama function name → MCP `(server, tool)` call.
- Name collision policy: prefix with server id (`nest_memory__search_project_memory`).

### 3.3 Read-only tool allowlist (v1)

Auto-run without approval:

| Server | Tool |
| --- | --- |
| `nest-memory` | `search_project_memory` |
| `nest-knowledge` | `search_knowledge_base`, `list_knowledge_collections` |
| `nest-context-memory` | `search_context_memory`, `list_context_memory`, `get_context_memory` |

Defer auto-run for `save_context_memory` to v1.1 (write).

### 3.4 System prompt

Inject a concise system message describing available tools and the multi-step loop
(Ollama docs recommend telling the model it may call tools repeatedly).

### 3.5 Tests

- Mock `AiProvider` returns tool_call then final text → mock tool executor
- Max steps exceeded → graceful error event
- Cancellation mid-loop → stop cleanly

**Deliverable:** `cargo test -p nest-agent`; CLI-only runner in Phase 4.

---

## Phase 4 — Kiwi integration

**Objective:** Agent mode in CLI first, then GUI.

### 4.1 Configuration (`apps/kiwi/desktop/config.toml`)

```toml
[agent]
host = "192.168.88.10"
port = 11434
model = "qwen2.5:7b"          # tool-capable default for agent mode
mcp_config = "../../.cursor/mcp.json"  # relative to config file
mcp_servers = ["nest-memory", "nest-knowledge", "nest-context-memory"]
max_steps = 10
```

### 4.2 CLI (first UX)

```bash
kiwi agent "What crates implement HTTP in nest-core?"
kiwi agent --stdin
```

- Reuse `nest-cli` + `TaskRuntimeModule`
- Construct `McpHub`, `AgentLoop`, print tool steps to stderr or structured log
- Exit code on failure

### 4.3 GUI

| Area | Change |
| --- | --- |
| `chat.rs` | `spawn_agent_run` emitting `AgentEvent` (parallel to stream events) |
| `workbench/mod.rs` | Toggle or separate "Agent" send mode; render tool call blocks in conversation |
| `sidebar/agent.rs` | MCP server status (connected / tool count), link to MCP-SETUP docs |
| `state.rs` | `ChatEntry` variant for tool steps, or separate `agent_steps` log |

**Chat UI sketch**

```text
You: What is nest-task?

🔧 search_project_memory({"query":"nest-task"})
   ↳ 3 snippets from docs/plan/nest-task-v1.md …

Kiwi: nest-task is the execution engine for …
```

### 4.4 Module wiring

```rust
// apps/kiwi/desktop/crates/kiwi/src/modules.rs
app.module(McpModule::new())   // optional
    .module(OllamaModule::new())
    .module(TaskRuntimeModule::owned(...))
```

**Deliverable:** End-to-end agent query using live `nest-memory` MCP from Kiwi CLI;
GUI shows tool steps for the same path.

---

## Phase 5 — Hardening (v1 polish)

| Item | Status |
| --- | --- |
| Agent response streaming (`stream_complete` + `TextDelta`) | **Done** |
| Parallel tool calls (`SharedMcpHub`) | **Done** |
| MCP hub reconnect on transport failure | **Done** |
| Agent sidebar: enable/disable MCP servers | **Done** |
| Optional `save_context_memory` auto-run | **Done** |
| Attached-file summarize + tool-call parsing fixes | **Done** |

---

## Testing strategy

| Layer | Approach |
| --- | --- |
| `nest-mcp` | Unit + mock stdio; ignored live test with Python server |
| `nest-ai` / `nest-ai-ollama` | Wiremock HTTP fixtures |
| `nest-agent` | Mock provider + mock tool registry |
| Kiwi | CLI integration test (ignored without Ollama + MCP env) |
| Manual QA checklist | See below |

### Manual QA checklist

1. PostgreSQL + indexed memory running ([MCP-SETUP.md](../../tools/MCP-SETUP.md))
2. Ollama running with tool-capable model
3. `kiwi agent "Search project memory for nest-tauri desktop platform"` shows tool call + answer
4. Kiwi GUI agent mode shows tool block + streaming final answer
5. Disabled MCP server → clear error in chat
6. Chat mode (non-agent) unchanged — no regression

---

## Dependencies (new crates)

```toml
# nest-mcp (illustrative)
tokio = { workspace = true, features = ["process", "io-util", "sync"] }
serde = { workspace = true }
serde_json = { workspace = true }
nest-error = { workspace = true }
tracing = { workspace = true }

# nest-agent
nest-ai = { workspace = true }
nest-error = { workspace = true }
async-trait = { workspace = true }
tokio = { workspace = true }
```

Add to root `Cargo.toml` workspace members + `[workspace.dependencies]`.

---

## Risks and mitigations

| Risk | Mitigation |
| --- | --- |
| Small models fail to call tools reliably | Document minimum model; agent mode warns if model is below recommended size |
| FastMCP stdio framing differs from spec | Phase 0 spike; adapt line reader |
| Memory MCP needs OpenAI key + Postgres | Surface clear errors in Agent sidebar; link MCP-SETUP |
| Path resolution when Kiwi cwd ≠ repo root | Resolve `mcp_config` relative to config file / explicit `nest_root` setting |
| Agent loop runaway | `max_steps` cap + cancellation on panel close |
| Security (future write tools) | v1 read-only auto-run only |

---

## Implementation order (summary)

```text
Phase 0  Spike Ollama tools + FastMCP stdio framing
Phase 1  nest-mcp (list + call)
Phase 2  nest-ai tool types + nest-ai-ollama tools API
Phase 3  nest-agent loop + events
Phase 4  Kiwi CLI agent → GUI tool UI
Phase 5  Hardening
```

Estimated sequencing: **Phases 1–3** are Nest framework work (usable from any app).
**Phase 4** is Kiwi-specific. CLI before GUI keeps feedback loops fast.

---

## Documentation to add on implementation

| Path | Purpose |
| --- | --- |
| [docs/nest-mcp/README.md](../nest-mcp/README.md) | MCP client crate overview |
| [docs/nest-agent/README.md](../nest-agent/README.md) | Agent loop API |
| Update [docs/nest-ai/README.md](../nest-ai/README.md) | Tool calling section |
| Update [tools/MCP-SETUP.md](../../tools/MCP-SETUP.md) | "Using MCP from Kiwi" section |

Re-index after doc merge:

```bash
./scripts/index-memory.sh
```

---

## Follow-up (v1.1+)

- `nest-mcp-module` for standard Nest app registration
- Tool approval UI and write-tool policies
- OpenAI / Anthropic tool calling providers
- HTTP MCP transport
- Native Rust memory search (bypass Python MCP for latency)
- Agent run persistence and transcript export
- GitNexus and third-party MCP servers from user config

---

## Related

- [architecture.md](../architecture.md) — layer rules
- [nest-ai README](../nest-ai/README.md) — inference contracts
- [nest-ai-ollama README](../nest-ai-ollama/README.md) — Ollama adapter
- [MCP-SETUP.md](../../tools/MCP-SETUP.md) — existing Python MCP servers
- [nest-task v1 plan](./nest-task-v1.md) — background task pattern for agent runs
- Kiwi app — `apps/kiwi/desktop/` (separate repo checkout)
