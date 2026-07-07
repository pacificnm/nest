# Swift — AI assistant

**Status:** Planned

## Scope

In-app **agent panel** using [`nest-agent`](../../../../docs/nest-agent/README.md), [`nest-ai-ollama`](../../../../docs/nest-ai-ollama/README.md), MCP servers, and Swift-native tools. Primary retrieval for project context is **vector search** over [`knowledge_items`](data-model.md) (notes, emails, Slack, docs).

## Requirements

### UX

| Element | Behavior |
|---------|----------|
| **Agent rail** | Collapsible right panel; persists width |
| **Chat thread** | User messages + assistant replies + tool step blocks |
| **Model picker** | List Ollama models from config; default in `config.toml` |
| **Context chips** | Show focus project, optional attached task/knowledge item |
| **Cancel** | Stop in-flight agent run |

Tool steps show: tool name, truncated args, result preview, duration.

### LLM and embeddings

- **Chat:** Ollama via `nest-ai-ollama` (tool calling)
- **Embeddings:** **Ollama by default** — same `[ollama].base_url`, `POST /api/embed` (e.g. `nomic-embed-text`, 768 dims)
- Optional **`[embeddings] provider = "openai"`** for cloud embeddings (requires API key; use `dimensions = 1536` and re-index)
- Config: `[agent]`, `[ollama]`, `[embeddings]` (`provider`, `model`, `dimensions`)

### MCP (optional)

Swift’s agent gets **project context from native tools** (`swift_search_knowledge`, task tools) over PostgreSQL — that is the main path.

**MCP is optional** for wiring extra stdio tool servers (custom integrations, future email/Slack MCP, etc.). Swift reads its **own** `mcp.json` next to `config.toml`, **not** Cursor’s `.cursor/mcp.json`.

| Source | Role in Swift |
|--------|----------------|
| **Swift-native tools** | Primary — vector search, tasks, notes |
| **`mcp.json`** | Optional extensions (`[agent.mcp] enabled = true`) |

Nest dev tools (`nest-memory`, `nest-knowledge`, `nest-context-memory`) belong in **Cursor** or a dev-only `mcp.json` if you want the in-app agent to search Nest framework docs while building Swift itself. End users do not need them.

Session key for context memory (if enabled): `swift:{project_slug}`.

### Swift-native tools

| Tool | Auto-run | Description |
|------|----------|-------------|
| `swift_search_knowledge` | yes | **Vector + keyword** over `knowledge_items`; default focus project, optional workspace-wide |
| `swift_search_tasks` | yes | Keyword/filter on tasks |
| `swift_get_knowledge_item` | yes | Full body + metadata by id |
| `swift_get_task` | yes | Task by id |
| `swift_list_projects` | yes | Active projects |
| `swift_create_task` | no* | Create task |
| `swift_update_task` | no* | Update task |
| `swift_create_note` | no* | Create note (`kind=note`) + index |
| `swift_update_note` | no* | Update note + re-index |

\* Requires `allow_writes = true`.

`swift_search_knowledge` is the main tool for “what did we decide about X?”, “summarize Slack thread”, “find email about Y” once ingest sources exist.

### System prompt context

Inject: focus project name/slug (if any), available knowledge kinds, write policy, current date.

### IPC

- Agent runs on Tokio async runtime in `src-tauri/`
- Events streamed via Tauri (see [ipc-api](ipc-api.md))

## Non-goals (v1)

- Multiple concurrent agent threads
- Cloud chat LLM providers (Ollama for chat + embeddings; OpenAI embeddings optional)
- Agent run history persistence

## Related plans

- [swift-agent-v1](../plan/swift-agent-v1.md)
- [nest-agent-mcp-v1](../../../../docs/plan/nest-agent-mcp-v1.md)
