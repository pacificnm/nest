# nest-data-postgres v1 Implementation Plan

## Status: Implemented

First PostgreSQL provider for [`nest-data`](../nest-data/README.md). Required by [Swift](../apps/swift/docs/README.md) for project-scoped **vector search** over knowledge (notes, emails, Slack, documentation).

## Context

[`nest-data-sqlite`](../nest-data-sqlite/README.md) covers sync local SQLite. Swift and future server-side apps need:

- **PostgreSQL** as the primary store
- **pgvector** for embedding search (same pattern as Nest MCP `project_memory` / `knowledge_base`)
- **Async I/O** via sqlx + Tokio (desktop Swift uses async through `nest-tauri` + Tokio)

## Crate boundaries

| Crate | Role |
|-------|------|
| `nest-data` | Contracts (`AsyncRepository`, `Migration`, `DataService`) — enable `async` feature |
| **`nest-data-postgres`** | sqlx/postgres driver, pool, migrations, pgvector helpers |
| Apps (Swift) | `swift-data` repositories on top of `PostgresDataModule` |

Location: `modules/crates/nest-data-postgres/`

## Dependencies (illustrative)

```toml
[dependencies]
nest-data = { workspace = true, features = ["async"] }
nest-core = { workspace = true }
nest-error = { workspace = true }
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "uuid", "chrono", "json"] }
tokio = { workspace = true }
pgvector = { version = "0.4", features = ["sqlx"] }
```

## Public API (v1)

| Type | Role |
|------|------|
| `PostgresConfig` | `DATABASE_URL`, pool size, schema |
| `PostgresDataModule` | Registers `DataService` with `PgPool` |
| `PostgresMigrationRunner` | Applies versioned SQL migrations |
| `PostgresConnection` | `DataConnection` + health check |
| `VectorSearch` helper | `search_similar(embedding, limit, project_id?)` — optional trait in v1 |

Register module id: `ModuleId("nest-data-postgres")`.

## Migrations

- `_nest_migrations` table (same contract as SQLite runner)
- Migration files: `migrations/*.sql` loaded by runner
- Document `CREATE EXTENSION vector` in setup (requires superuser once)

## Vector search

| Concern | Approach |
|---------|----------|
| Storage | `vector(N)` column via pgvector; N matches embedding model (default **1536** for OpenAI `text-embedding-3-small`) |
| Index | `ivfflat` or `hnsw` on `(project_id, embedding)` — start with sequential scan + limit for v1 dev |
| Hybrid | Optional `tsvector` + vector rerank (Swift v1.1) |

Embedding generation stays **out of nest-data-postgres** — apps call `nest-ai` or a small `EmbeddingService`; postgres crate stores/query vectors only.

## Phases

| Phase | Deliverable |
|-------|-------------|
| 1a | Crate scaffold, `PostgresConfig`, pool connect, health check |
| 1b | `PostgresMigrationRunner` + sample migration |
| 1c | `AsyncRepository` example (notes or generic row) |
| 1d | pgvector column type + `similarity_search` helper |
| 1e | Docs, integration test (ignored without `DATABASE_URL`) |

## Swift dependency

Swift [swift-data-v1](../apps/swift/docs/plan/swift-data-v1.md) **blocks** on nest-data-postgres **1a–1d** (pool, migrations, vector helper).

## Setup (dev)

```bash
sudo apt install postgresql postgresql-contrib postgresql-XX-pgvector
createdb swift
psql swift -c "CREATE EXTENSION vector;"
```

Config in Swift `config.toml`:

```toml
[database]
url = "postgresql:///swift?host=/var/run/postgresql"
```

## Non-goals (v1)

- SQLx compile-time query checking in CI (use runtime queries or checked-in `.sqlx` cache later)
- Multi-tenant RLS
- Connection pooling across processes
- Replacing Nest MCP memory database (separate `nest_memory` DB remains)

## Done when

1. `PostgresDataModule` registers pool in `DataService`
2. Migrations apply on empty DB with pgvector enabled
3. Integration test inserts rows and runs vector similarity query
4. [swift-data-v1](../apps/swift/docs/plan/swift-data-v1.md) can depend on the crate

## Related

- [nest-data v1](./nest-data-v1.md)
- [Swift data model](../apps/swift/docs/specs/data-model.md)
- [MCP-SETUP pgvector](../../tools/MCP-SETUP.md)
