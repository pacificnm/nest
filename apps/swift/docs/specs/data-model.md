# Swift — data model

**Status:** Planned

## Scope

PostgreSQL persistence via [`nest-data`](../../../../docs/nest-data/README.md) and **[`nest-data-postgres`](../../../../docs/nest-data-postgres/README.md)** (to be built). **pgvector** enables per-project semantic search for the AI assistant.

## Storage

| Setting | Value |
|---------|-------|
| Engine | PostgreSQL 15+ |
| Extension | **pgvector** |
| Database | `swift` (default; configurable via `[database].url`) |
| Migrations | `_nest_migrations` via `PostgresMigrationRunner` |
| Pool | sqlx + Tokio (async) |

Swift PM tables (projects, tasks) and knowledge tables share one database. Vector index is **scoped by `project_id`**.

## Tables (v1)

### Project management

```text
projects          (id, slug, name, description, color, icon, archived, sort_order, created_at, updated_at)
tasks             (id, project_id, parent_id, title, description, status, priority, due_date, sort_order,
                   created_at, updated_at, completed_at)
labels            (id, project_id, name, color)
task_labels       (task_id, label_id)
task_notes        (task_id, knowledge_item_id)   -- link tasks to knowledge rows
```

### Knowledge (unified searchable content)

All content the AI must search — notes, emails, Slack messages, imported docs — lives in **`knowledge_items`**:

```text
knowledge_items   (id, project_id, kind, title, body, metadata_json,
                   source_uri, source_external_id,
                   embedding vector(1536),     -- pgvector; null until indexed
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
| Scope | Every similarity query filters `WHERE project_id = $active` (optional `NULL` for workspace-wide) |
| Embedding model | Configurable; default OpenAI `text-embedding-3-small` (1536 dims) — generation in app layer, not in nest-data-postgres |
| Re-index | On create/update of `body`, enqueue re-embed; store `indexed_at` |
| Agent access | Via `swift_search_knowledge` tool (vector + optional keyword) |

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
- Cross-project vector search in agent tools (workspace search deferred)

## Related plans

- [swift-data-v1](../plan/swift-data-v1.md)
- [nest-data-postgres v1](../../../../docs/plan/nest-data-postgres-v1.md)
