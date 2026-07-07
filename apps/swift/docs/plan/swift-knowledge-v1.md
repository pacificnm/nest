# Swift knowledge v1

## Status: Planned

Phase **3** of [swift-v1](./swift-v1.md). Knowledge UI, vector search, note authoring, and embedding index.

## Specs

- [knowledge](../specs/knowledge.md)
- [data-model](../specs/data-model.md)
- [ipc-api](../specs/ipc-api.md)

## Prerequisites

- [swift-data-v1](./swift-data-v1.md) — `knowledge_items` + pgvector
- [nest-data-postgres v1](../../../docs/plan/nest-data-postgres-v1.md) — complete
- [swift-pm-v1](./swift-pm-v1.md) — project context

## Rust

| Service | Role |
|---------|------|
| `KnowledgeService` | CRUD, re-index embeddings, `search` (vector + keyword) |
| `EmbeddingService` | Ollama `/api/embed` on save (extend `nest-ai-ollama` or thin HTTP client) |

### Tauri commands

- `swift_list_knowledge` — filter by `kind`, project
- `swift_get_knowledge_item`
- `swift_create_note` / `swift_update_note` — `kind=note`
- `swift_import_doc` — text/markdown file → `kind=doc`
- `swift_search_knowledge` — `{ query, project_id?, kind?, mode: vector|keyword|hybrid }`

## React

| Route | View |
|-------|------|
| `/p/:slug/knowledge` | All items by kind + search |
| `/p/:slug/notes/:id` | Note editor (notes subset) |

### Search UX

- Semantic search bar → `swift_search_knowledge` (vector)
- Toggle keyword mode for exact matches
- Show kind badges: note, doc, email, slack

## Phases

| Step | Deliverable |
|------|-------------|
| 3a | Knowledge list + note editor |
| 3b | Embedding on save + vector search API |
| 3c | In-app semantic search UI |
| 3d | Doc import + task linking |
| 3e | Email/Slack ingest stubs (schema ready; importers v1.1) |

## Done when

- User authors notes; content is vector-indexed per project
- Semantic search returns relevant notes/docs
- Agent can call `swift_search_knowledge` with same backend

## Related

- [swift-agent-v1](./swift-agent-v1.md)
- [swift-data-v1](./swift-data-v1.md)
