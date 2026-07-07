# Swift agent v1

## Status: Planned

Phase **4** of [swift-v1](./swift-v1.md). Ollama AI assistant with MCP and Swift-native tools. Primary project context comes from **vector search** over `knowledge_items`.

## Specs

- [ai-assistant](../specs/ai-assistant.md)
- [ipc-api](../specs/ipc-api.md) — agent commands + events

## Prerequisites

- [swift-pm-v1](./swift-pm-v1.md) — task data for tools
- [swift-knowledge-v1](./swift-knowledge-v1.md) — embeddings + `KnowledgeRepository::similarity_search`
- [nest-agent-mcp-v1](../../../docs/plan/nest-agent-mcp-v1.md) — patterns implemented in nest

## Rust (`src-tauri/`)

### Modules

| Module | Crates |
|--------|--------|
| AI | `nest-ai`, `nest-ai-ollama`, `AiModule` |
| Agent | `nest-agent`, `nest-mcp` |
| Swift tools | `swift-agent-tools` (new crate in `apps/swift/crates/`) |

Enable `nest-tauri` feature **`async`** for Tokio agent runs.

### Tool sources

`CompositeToolSource`:

1. **MCP hub** (optional) — load from Swift **`mcp.json`** beside `config.toml` when `[agent.mcp] enabled = true`; not Cursor config
2. **SwiftToolSource** — in-process: vector knowledge search, task search/get, gated writes (primary)

Implement [ai-assistant tool table](../specs/ai-assistant.md).

Key tool: **`swift_search_knowledge`** — embeds the query, runs pgvector similarity search scoped to focus project (or workspace-wide when `project_id` omitted), optional `kind` filter.

### Agent runner

- Background Tokio task per `swift_agent_start`
- Map `AgentEvent` → Tauri events (`swift://agent/*`)
- `swift_agent_cancel` sets cancel token

### Config

```toml
[agent]
max_steps = 8
tool_timeout_secs = 60
allow_writes = false
allow_file_writes = false

[agent.mcp]
enabled = false
config_path = "mcp.json"

[embeddings]
provider = "ollama"
model = "nomic-embed-text"
dimensions = 768
```

## React

| Component | Behavior |
|-----------|----------|
| `AgentPanel` | Chat input, message list, model picker |
| `ToolStepBlock` | Collapsible tool call UI |
| `useAgentRun` | subscribe to Tauri events for `run_id` |

Wire into agent rail from [ui-platform](../specs/ui-platform.md).

## Phases

| Step | Deliverable |
|------|-------------|
| 4a | Ollama chat without tools (smoke) |
| 4b | MCP hub + memory search tools |
| 4c | Swift read tools (`swift_search_knowledge`, search/get tasks) |
| 4d | Write tools behind `allow_writes` |
| 4e | Streaming text + tool UI polish |

## Tests

- `swift-agent-tools` unit tests with mock `KnowledgeRepository` (vector search returns ordered hits)
- Ignored integration test: requires Ollama + MCP + PostgreSQL

## Done when

- User asks agent to search project knowledge semantically (notes, docs)
- Agent uses MCP for Nest framework docs when relevant
- With `allow_writes=true`, agent can create a task from natural language
- Tool steps visible in agent panel

## Related

- [nest-agent docs](../../../docs/nest-agent/README.md)
- [swift-v1](./swift-v1.md) phase 6 — activity logging for agent queries
