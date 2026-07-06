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
| **Context chips** | Show active project, optional attached task/knowledge item |
| **Cancel** | Stop in-flight agent run |

Tool steps show: tool name, truncated args, result preview, duration.

### LLM

- Provider: **Ollama** via `nest-ai-ollama` (chat + tool calling)
- Embedding provider: configurable (default OpenAI API for `text-embedding-3-small`) — used by knowledge indexing, not necessarily the chat model
- Config: `[agent]`, `[ollama]`, `[embeddings]`

### MCP tools (read-heavy, auto-run)

| Server | Tools |
|--------|-------|
| `nest-memory` | `search_project_memory` |
| `nest-knowledge` | `search_knowledge_base` |
| `nest-context-memory` | `search_context_memory`, `save_context_memory` |

Session key: `swift:{project_slug}`.

### Swift-native tools

| Tool | Auto-run | Description |
|------|----------|-------------|
| `swift_search_knowledge` | yes | **Vector + keyword** search over `knowledge_items` in active project; filter by `kind` optional |
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

Inject: active project name/slug, available knowledge kinds, write policy, current date.

### IPC

- Agent runs on Tokio async runtime in `src-tauri/`
- Events streamed via Tauri (see [ipc-api](ipc-api.md))

## Non-goals (v1)

- Multiple concurrent agent threads
- Cloud chat LLM providers (embeddings API OK)
- Agent run history persistence

## Related plans

- [swift-agent-v1](../plan/swift-agent-v1.md)
- [nest-agent-mcp-v1](../../../../docs/plan/nest-agent-mcp-v1.md)
