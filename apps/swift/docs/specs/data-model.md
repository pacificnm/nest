# Swift — data model

**Status:** Designed — see [database-schema](database-schema.md) for column-level detail and SQL migrations.

## Scope

PostgreSQL persistence via [`nest-data`](../../../../docs/nest-data/README.md) and
[`nest-data-postgres`](../../../../docs/nest-data-postgres/README.md). **pgvector**
enables per-project semantic search for the AI assistant.

## Storage

| Setting | Value |
|---------|-------|
| Engine | PostgreSQL 15+ |
| Extension | **pgvector**, **pgcrypto** |
| Host | **Remote server** (LAN/VPN); not embedded in the desktop app |
| Database | `swift` (configurable via `[database].url` or `DATABASE_URL`) |
| Migrations | `apps/swift/crates/swift-data/migrations/` → `_nest_migrations` |
| Pool | sqlx + Tokio (async) |

Swift connects to PostgreSQL over the network (same pattern as Ollama on `server.lan`).

## Tables (v1)

Aligned with UI mock types in `ui/src/mock/demo.ts`, `knowledgeDemo.ts`, `settingsDemo.ts`.

```text
calendars              -- Working Time (Standard / 24h presets)
projects               -- MockProject + Project Info fields
tasks                  -- MockTask (Gantt, task form)
task_links             -- predecessors (FS default)
knowledge_categories   -- MockKnowledgeCategory
knowledge_articles     -- MockKnowledgeArticle + pgvector embedding
knowledge_revisions    -- MockKnowledgeRevision (append-only)
task_knowledge_links   -- task ↔ article (optional v1)
app_settings           -- AppSettings sections (file, task, view, …)
```

### Project management

```text
projects          (id, slug, name, description, color, icon, manager,
                   archived, pinned, percent_complete,
                   start_date, finish_date, status_date, priority,
                   calendar_id, sort_order, created_at, updated_at)

tasks             (id, project_id, parent_id, outline_level, is_summary, is_milestone,
                   title, notes, duration_days, duration_minutes,
                   start_date, finish_date, percent_complete, resource_names,
                   priority, constraint_type, constraint_date, deadline,
                   effort_driven, task_type, sort_order,
                   actual_start, actual_finish, created_at, updated_at)

task_links        (id, project_id, predecessor_id, successor_id, link_type, lag_minutes)
```

### Knowledge

```text
knowledge_categories   (id, project_id, name, description, sort_order, …)
knowledge_articles     (id, project_id, category_id, title, body,
                        source_type, source_label, source_uri,
                        embedding vector(768), search_text, indexed_at, …)
knowledge_revisions    (id, article_id, revision_number, title, body,
                        change_note, created_by, created_at)
```

| `source_type` | UI label | Agent kind |
|---------------|----------|------------|
| `manual` | Manual | `note` |
| `doc` | Document | `doc` |
| `email` | Email | `email` |
| `slack` | Slack | `slack` |
| `url` | Web | `doc` |

### App state

```text
app_settings      (key, value_json, updated_at)   -- keys: file, task, view, project, knowledge, database, agent
```

## Vector search

| Rule | Detail |
|------|--------|
| Scope | Default: `WHERE project_id = $focus`. Omit for workspace-wide search |
| Column | `knowledge_articles.embedding vector(768)` |
| Model | Ollama `nomic-embed-text` (768 dims) via `[embeddings]` |
| Re-index | On article save; `indexed_at` timestamp |
| Hybrid | `search_text` tsvector + vector cosine |

## Indexes (v1)

| Table | Index | Purpose |
|-------|-------|---------|
| `projects` | `(archived, pinned DESC, sort_order)` | Project Center library |
| `projects` | `slug` UNIQUE WHERE NOT archived | Routing |
| `tasks` | `(project_id, sort_order)` | Gantt row order |
| `knowledge_articles` | HNSW on `embedding` | Vector search |
| `knowledge_articles` | GIN on `search_text` | Keyword search |

## Repositories (Rust)

| Repository | Crate |
|------------|-------|
| `ProjectRepository`, `TaskRepository`, `TaskLinkRepository` | `swift-data` |
| `KnowledgeCategoryRepository`, `KnowledgeArticleRepository` | `swift-data` |
| `AppSettingsRepository` | `swift-data` |

Registered via `SwiftDataModule` → depends on `PostgresDataModule`.

## Cascades

| Action | Behavior |
|--------|----------|
| Archive project | Hide from UI; retain knowledge rows |
| Delete project | CASCADE tasks, categories, articles |
| Delete article | CASCADE revisions + task links |
| Delete task | CASCADE task_links; SET NULL on child `parent_id` |

## Non-goals (v1)

| Feature | Phase |
|---------|-------|
| `labels` / `task_labels` | v1.1 |
| `activity_events` / `timer_sessions` | phase 6 |
| SQLite / offline-only | — |

## Related

- [database-schema](database-schema.md) — full column mapping + ER diagram
- [swift-data-v1](../plan/swift-data-v1.md)
- [nest-data-postgres v1](../../../../docs/plan/nest-data-postgres-v1.md)
