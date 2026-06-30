# nest-data

Database-agnostic data contracts for the [Nest framework](../../README.md).

**Crate path:** [`crates/nest-data`](../../crates/nest-data)

## Role

nest-data defines **what data access means**. Provider crates (`nest-data-sqlite`, future `nest-data-postgres`) decide **how I/O happens**.

| Layer | Responsibility |
|-------|----------------|
| nest-core | Registers `DataService`; stays sync |
| nest-data | Contracts, errors, connection registry |
| nest-data-sqlite | rusqlite driver, migrations |
| nest-tasks / nest-api-serve (future) | Runs async repository calls |

## Quick start

```rust
use nest_core::AppBuilder;
use nest_data::{DataModule, DataService, DATA_MODULE_ID};
use nest_data_sqlite::SqliteDataModule;

AppBuilder::new()
    .module(DataModule)
    .module(SqliteDataModule::primary("app.db"))
    .build()?;
```

## Sync vs async

| Use case | API |
|----------|-----|
| Desktop / local SQLite | Sync `Repository` |
| CLI importer | Sync or async |
| API server / background jobs | `AsyncRepository` (feature `async`) |

Enable async traits:

```toml
nest-data = { path = "../nest-data", features = ["async"] }
```

nest-core does **not** execute async I/O. Future `nest-tasks` and `nest-api-serve` will run async repository calls.

## Core contracts

- **`Repository<TEntity, TId>`** — get, list, insert, update, delete
- **`Transactional`** — `begin()` → `Transaction` with commit/rollback
- **`Migration` / `MigrationRunner`** — versioned schema changes
- **`DataConnection`** — health check + metadata
- **`DataService`** — named connection registry (`primary`, `cache`, …)

## Module integration

```rust
pub const DATA_MODULE_ID: ModuleId = ModuleId("nest-data");
```

Provider modules declare `dependencies() -> &[DATA_MODULE_ID]` and register connections via `app.service_mut::<DataService>()?`.

## Related

- [nest-data-sqlite](../nest-data-sqlite/README.md) — first provider
- [nest-core modules](../nest-core/modules.md) — DI pattern
