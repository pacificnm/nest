# Swift — data model

**Status:** Planned

## Scope

PostgreSQL persistence via [`nest-data`](../../../../docs/nest-data/README.md) and **[`nest-data-postgres`](../../../../docs/nest-data-postgres/README.md)** (to be built). **pgvector** enables per-project semantic search for the AI assistant.

## Storage

| Setting | Value |
|---------|-------|
| Engine | PostgreSQL 15+ |
| Extension | **pgvector** |
| Host | **Remote server** (LAN/VPN); not embedded in the desktop app |
| Database | `swift` (configurable via `[database].url` or `DATABASE_URL`) |
| Migrations | `_nest_migrations` via `PostgresMigrationRunner` |
| Pool | sqlx + Tokio (async) |

Swift connects to PostgreSQL over the network (same pattern as Ollama on `server.lan`). No local Postgres install required on the workstation running the Tauri app.

Swift PM tables (projects, tasks) and knowledge tables share one database. Vector index is **scoped by `project_id`**.

## Tables (v1)

### Project management

```text
projects          (id, slug, name, description, color, icon, archived, pinned, sort_order, created_at, updated_at)
tasks             (id, project_id, parent_id, outline_level, is_summary, is_milestone,
                   title, duration_minutes, start_at, finish_at, percent_complete,
                   sort_order, created_at, updated_at, actual_start, actual_finish)
task_links        (id, predecessor_id, successor_id, link_type, lag_minutes)
labels            (id, project_id, name, color)
task_labels       (task_id, label_id)
task_notes        (task_id, knowledge_item_id)   -- link tasks to knowledge rows
```

### Knowledge (unified searchable content)

All content the AI must search — notes, emails, Slack messages, imported docs — lives in **`knowledge_items`**:

```text
knowledge_items   (id, project_id, kind, title, body, metadata_json,
                   source_uri, source_external_id,
                   embedding vector(768),      -- pgvector; dims match [embeddings].dimensions (default: nomic-embed-text)
                   search_text tsvector,       -- generated for keyword hybrid search
                   created_at, updated_at, indexed_at)
```

| `kind` | Examples |
|--------|----------|
| `note` | User markdown notes (editor UX) |
| `email` | Imported email body + headers in `metadata_json` |
| `slack` | Channel/DM message |
| `doc` | Project documentation, specs, attachments (text extracted) |

Optional structure for notes UI only:

```text
note_folders      (id, project_id, parent_id, name, sort_order)
-- notes with kind=note may reference folder_id via metadata_json or dedicated column (v1.1)
```

### Tracking

```text
activity_events   (id, event_type, payload_json, occurred_at)
timer_sessions    (id, task_id, started_at, ended_at, duration_ms)
```

### App state

```text
app_settings      (key, value_json)
```

## Vector search

| Rule | Detail |
|------|--------|
| Scope | Default: filter `WHERE project_id = $focus`. Omit `project_id` for **workspace-wide** search (agent + library search) |
| Embedding model | **Ollama** default (`nomic-embed-text`, **768** dims) via `[embeddings]` + same server as chat; optional OpenAI (`1536` dims) |
| Re-index | On create/update of `body`, enqueue re-embed; store `indexed_at`. **Re-index all rows** if embedding model or dimensions change |
| Agent access | Via `swift_search_knowledge` tool (vector + optional keyword); optional `project_id` |

## Indexes (v1)

Portfolio scale — hundreds of projects, tasks scoped per project:

| Table | Index | Purpose |
|-------|-------|---------|
| `projects` | `(archived, pinned DESC, sort_order)` | Library + sidebar daily set |
| `projects` | `slug` UNIQUE | Routing |
| `tasks` | `(project_id, status, sort_order)` | Kanban/list per project |
| `tasks` | `(project_id, due_date)` WHERE `completed_at IS NULL` | Due filters |
| `knowledge_items` | `(project_id, kind)` | Scoped lists |
| `knowledge_items` | HNSW or IVFFlat on `embedding` | Vector search (with `project_id` filter when set) |

## Repositories (Rust)

| Repository | Crate |
|------------|-------|
| `ProjectRepository`, `TaskRepository` | `apps/swift/crates/swift-data` |
| `KnowledgeRepository` | CRUD + `similarity_search`, `keyword_search`, ingest hooks |
| `ActivityRepository` | phase 6 |

Registered via `SwiftDataModule` → depends on `PostgresDataModule`.

## Cascades

| Action | Behavior |
|--------|----------|
| Archive project | Hide from UI; retain knowledge rows |
| Delete knowledge item | Remove embeddings + links |
| Delete task | Remove `task_notes` links only |

## Non-goals (v1)

- SQLite / offline-only mode
- Encryption at rest
- Full-text search across all tasks in UI (cross-project task inbox deferred to v1.1)

## Related plans

- [swift-data-v1](../plan/swift-data-v1.md)
- [nest-data-postgres v1](../../../../docs/plan/nest-data-postgres-v1.md)
