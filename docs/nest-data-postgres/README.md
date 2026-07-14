# nest-data-postgres

PostgreSQL provider for [`nest-data`](README.md) with **pgvector** support.

**Crate path:** `modules/crates/nest-data-postgres`

**Status:** Implemented (v1). See [implementation plan](../plan/nest-data-postgres-v1.md).

## Role

- Async connection pool (sqlx 0.9 + Tokio)
- Versioned SQL migrations via [`PostgresMigrationRunner`](../../modules/crates/nest-data-postgres/src/migration.rs)
- pgvector similarity search via [`VectorSearch`](../../modules/crates/nest-data-postgres/src/vector.rs)

## Quick start

```rust
use nest_core::AppBuilder;
use nest_data::{DataModule, DataService};
use nest_data_postgres::{PostgresConfig, PostgresDataModule};

AppBuilder::new()
    .module(DataModule)
    .module(PostgresDataModule::new(PostgresConfig::from_env("DATABASE_URL")?))
    .build()?;
```

Enable pgvector once per database:

```sql
CREATE EXTENSION IF NOT EXISTS vector;
```

Or register [`enable_vector_migration()`](../../modules/crates/nest-data-postgres/src/vector.rs) with the module.

## Vector search

```rust
use nest_data_postgres::VectorSearch;

let search = VectorSearch::new(pool, "knowledge_items", "id", "embedding")
    .with_project_scope("project_id");

let hits = search
    .search_similar(&query_embedding, 10, Some("project-uuid"))
    .await?;
```

## Tests

All tests run automatically against disposable `testcontainers`-managed PostgreSQL
(and pgvector, for the vector search test) — no manual database setup required:

```bash
cargo test -p nest-data-postgres
```

## Related

- [nest-data](README.md)
- [nest-data-sqlite](../nest-data-sqlite/README.md)
- [nest-data-postgres v1 plan](../plan/nest-data-postgres-v1.md)
