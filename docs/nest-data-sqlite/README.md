# nest-data-sqlite

SQLite provider for the Nest data layer.

**Crate path:** [`modules/crates/nest-data-sqlite`](../../modules/crates/nest-data-sqlite)

## Quick start

```rust
use nest_core::AppBuilder;
use nest_data::DataModule;
use nest_data_sqlite::{SqliteDataModule, notes_migration};

AppBuilder::new()
    .module(DataModule)
    .module(
        SqliteDataModule::primary("app.db")
            .with_migration(Box::new(notes_migration())),
    )
    .build()?;
```

## In-memory (tests)

```rust
SqliteDataModule::memory().with_migration(Box::new(notes_migration()))
```

## Named connections

```rust
SqliteDataModule::named("cache", "cache.db")
```

Register additional connections in your own modules via `DataService::register_connection`.

## Migrations

Migrations use the `_nest_migrations` table. Define migrations with [`nest_data::SqlMigration`](../../core/crates/nest-data/src/migration.rs) or custom `Migration` impls.

```rust
nest_data::SqlMigration::new(
    "001_create_notes",
    "CREATE TABLE notes (...);",
    "DROP TABLE notes;",
)
```

## Repository pattern

Implement `nest_data::Repository` in your app using `SqliteConnection`:

```rust
struct NotesRepository {
    db: SqliteConnection,
}

impl Repository<Note, NoteId> for NotesRepository {
    fn get(&self, id: NoteId) -> DataResult<Option<Note>> { /* ... */ }
    // ...
}
```

See [`NotesRepository`](../../modules/crates/nest-data-sqlite/src/notes.rs) for a full example.

## Features

| Feature | Description |
|---------|-------------|
| `sync` (default) | rusqlite sync driver |
| `async` | Reserved for future sqlx + tokio (not implemented in v1) |

## Related

- [nest-data](../nest-data/README.md) — contracts
