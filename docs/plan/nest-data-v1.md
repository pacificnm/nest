# nest-data v1 Implementation Plan

## Status: Implemented

See [nest-data docs](../nest-data/README.md) and [nest-data-sqlite docs](../nest-data-sqlite/README.md).

## Context

Database-agnostic data contract layer. nest-data defines what data access means; provider crates implement how I/O happens. Sync traits always available; async behind `async` feature.

## Crate boundaries

| Crate | Role |
|-------|------|
| `nest-data` | `Repository`, `Transaction`, `Migration`, `DataService`, `DataModule` |
| `nest-data-sqlite` | Sync SQLite via rusqlite, `SqliteDataModule`, example `NotesRepository` |
| `nest-core` | Unchanged — registers services, does not run DB I/O |

## nest-data

- `DataModule` registers `DataService` (connection registry + active id)
- Sync `Repository`, `Transaction`, `Transactional`, `MigrationRunner`
- `AsyncRepository` / `AsyncTransaction` behind `async` feature
- `DataError` + `From<DataError> for NestError`

## nest-data-sqlite

- `SqliteDataModule` depends on `DATA_MODULE_ID`
- `SqliteConnection`: `DataConnection`, `Transactional`, health check
- `_nest_migrations` table + `SqliteMigrationRunner`
- Example `NotesRepository` proving `Repository` contract

## v1 limitations

- No `nest-data-sqlx` shared layer (postgres uses sqlx directly)
- No `nest-data-sqlite/async` implementation (feature gate only for future)
- Minimal `ListQuery` (limit/offset only)
- No trait-object repository lookup in nest-core

## Providers

| Crate | Status |
|-------|--------|
| `nest-data-sqlite` | Sync SQLite (implemented) |
| **`nest-data-postgres`** | Async PostgreSQL + pgvector (implemented) |

## Follow-up

- Extract `nest-data-sqlx` when a second SQL provider needs shared sqlx helpers
- `nest-data-sqlite/async` via sqlx + tokio
- Richer query/filter contracts
