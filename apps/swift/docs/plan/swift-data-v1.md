# Swift data v1

## Status: Planned

Phase **1** of [swift-v1](./swift-v1.md). PostgreSQL persistence, pgvector knowledge index, and Rust repositories via **`nest-data-postgres`**.

## Context

Implement [data-model spec](../specs/data-model.md) using:

- [`nest-data`](../../../core/crates/nest-data) (async)
- **[`nest-data-postgres`](../../../docs/nest-data-postgres/README.md)** — **must be built first** ([nest-data-postgres v1](../../../docs/plan/nest-data-postgres-v1.md))

## Prerequisites

| Dependency | Plan |
|------------|------|
| `nest-data-postgres` crate | [nest-data-postgres-v1](../../../docs/plan/nest-data-postgres-v1.md) phases 1a–1d |

## Boundaries

| In scope | Out of scope |
|----------|--------------|
| `SwiftDataModule`, Postgres migrations | Email/Slack ingest (v1.1) |
| PM tables + `knowledge_items` + pgvector | Full IPC CRUD (phase 2–3) |
| `KnowledgeRepository::similarity_search` | Agent tools (phase 4) |
| Embedding enqueue on note save (stub OK) | Activity/timer (phase 6) |

## Crate: `swift-data`

Location: `apps/swift/crates/swift-data/`

| Component | Role |
|-----------|------|
| `SwiftDataModule` | Registers repos on `PostgresDataModule` pool |
| `migrations/` | SQL for PM + knowledge + pgvector |
| `project.rs`, `task.rs` | PM repositories |
| `knowledge.rs` | `knowledge_items` CRUD + vector/keyword search |
| `embedding.rs` | Call embedding API; update `embedding` column |

Depends on: `nest-data`, `nest-data-postgres`, `nest-core`, `nest-error`, `tokio`.

## Migrations (v1)

1. `001_enable_vector.sql` — `CREATE EXTENSION IF NOT EXISTS vector`
2. `002_projects_tasks.sql` — includes `pinned`, indexes on `projects` and `tasks`
3. `003_knowledge_items.sql` — includes `vector(768)` (or `[embeddings].dimensions`), `tsvector`, indexes
4. `004_app_settings.sql`

## Phases

### Phase 1a — Postgres module wiring

- `PostgresDataModule` in `main.rs` from `[database].url`
- `swift_db_health` command

### Phase 1b — PM repositories

- `ProjectRepository`, `TaskRepository` + tests (testcontainers or `DATABASE_URL`)

### Phase 1c — Knowledge + vectors

- `KnowledgeRepository`: insert, update, `similarity_search(project_id, embedding, limit, kind?)`
- Keyword search via `search_text`

### Phase 1d — Embedding pipeline

- On note/doc save: compute embedding, upsert vector
- Config `[embeddings]` model + API key

## Config

```toml
[database]
# Remote server — pgvector enabled on the `swift` database
url = "postgresql://swift:CHANGE_ME@server.lan:5432/swift"
```

Or set `DATABASE_URL` in the environment (takes precedence when wired in phase 1).

Server setup (once): create database `swift`, run `CREATE EXTENSION vector`, grant the app user connect + DDL for migrations.

```toml
[embeddings]
provider = "ollama"
model = "nomic-embed-text"
dimensions = 768
# Uses [ollama].base_url — no API key
```

Optional OpenAI: `provider = "openai"`, `model = "text-embedding-3-small"`, `dimensions = 1536`, `OPENAI_API_KEY` in env.

Server: `ollama pull nomic-embed-text` (or your chosen embed model). Migration `vector(dimensions)` must match config.

## Tests

- `cargo test -p swift-data` with `DATABASE_URL` (ignored in CI without Postgres)
- Vector search round-trip: insert two items, query returns nearest

## Done when

- All tables from [data-model](../specs/data-model.md) exist in PostgreSQL
- pgvector similarity search works per project
- Repositories covered by tests

## Related

- [nest-data-postgres v1](../../../docs/plan/nest-data-postgres-v1.md)
- [swift-scaffold-v1](./swift-scaffold-v1.md)
- [swift-pm-v1](./swift-pm-v1.md)
