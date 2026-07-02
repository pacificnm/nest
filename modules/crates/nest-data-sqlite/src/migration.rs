//! SQLite migration runner.

use nest_data::{DataError, DataResult, Migration, MigrationRunner};

use crate::connection::SqliteConnection;
use crate::error::sqlite_result;

const MIGRATIONS_TABLE: &str = "_nest_migrations";

fn ensure_table(conn: &SqliteConnection) -> DataResult<()> {
    conn.with_connection(|db| {
        sqlite_result(db.execute(
            &format!(
                "CREATE TABLE IF NOT EXISTS {MIGRATIONS_TABLE} (
                    id TEXT PRIMARY KEY NOT NULL,
                    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
                )"
            ),
            [],
        ))?;
        Ok(())
    })
}

fn applied_ids(conn: &SqliteConnection) -> DataResult<Vec<String>> {
    ensure_table(conn)?;
    conn.with_connection(|db| {
        let mut stmt = sqlite_result(db.prepare(&format!(
            "SELECT id FROM {MIGRATIONS_TABLE} ORDER BY applied_at ASC"
        )))?;
        let rows = sqlite_result(stmt.query_map([], |row| row.get::<_, String>(0)))?;
        let mut ids = Vec::new();
        for row in rows {
            ids.push(sqlite_result(row)?);
        }
        Ok(ids)
    })
}

fn pending_ids(
    conn: &SqliteConnection,
    migrations: &[Box<dyn Migration>],
) -> DataResult<Vec<String>> {
    let applied: std::collections::HashSet<String> = applied_ids(conn)?.into_iter().collect();
    Ok(migrations
        .iter()
        .map(|m| m.id().to_string())
        .filter(|id| !applied.contains(id))
        .collect())
}

/// Applies pending migrations to a SQLite connection.
pub fn apply_migrations(
    conn: &SqliteConnection,
    migrations: &[Box<dyn Migration>],
) -> DataResult<()> {
    let pending = pending_ids(conn, migrations)?;
    for migration in migrations {
        if !pending.iter().any(|id| id == migration.id()) {
            continue;
        }
        conn.with_connection(|db| {
            sqlite_result(db.execute_batch(migration.up_sql()))?;
            sqlite_result(db.execute(
                &format!("INSERT INTO {MIGRATIONS_TABLE} (id) VALUES (?1)"),
                [migration.id()],
            ))
        })
        .map_err(|error| {
            DataError::migration(format!(
                "failed to apply migration `{}`: {error}",
                migration.id()
            ))
        })?;
    }
    Ok(())
}

/// Rolls back the most recently applied migration.
pub fn rollback_last_migration(
    conn: &SqliteConnection,
    migrations: &[Box<dyn Migration>],
) -> DataResult<()> {
    let applied = applied_ids(conn)?;
    let last_id = applied
        .last()
        .ok_or_else(|| DataError::migration("no migrations to roll back"))?;

    let migration = migrations
        .iter()
        .find(|m| m.id() == last_id)
        .ok_or_else(|| DataError::migration(format!("migration `{last_id}` is not registered")))?;

    conn.with_connection(|db| {
        sqlite_result(db.execute_batch(migration.down_sql()))?;
        sqlite_result(db.execute(
            &format!("DELETE FROM {MIGRATIONS_TABLE} WHERE id = ?1"),
            [migration.id()],
        ))
    })?;

    Ok(())
}

/// Runs migrations against a SQLite connection.
pub struct SqliteMigrationRunner {
    conn: SqliteConnection,
    migrations: Vec<Box<dyn Migration>>,
}

impl SqliteMigrationRunner {
    /// Creates a runner with the given migrations.
    pub fn new(conn: SqliteConnection, migrations: Vec<Box<dyn Migration>>) -> Self {
        Self { conn, migrations }
    }
}

impl MigrationRunner for SqliteMigrationRunner {
    fn pending(&self) -> DataResult<Vec<String>> {
        pending_ids(&self.conn, &self.migrations)
    }

    fn apply_all(&self) -> DataResult<()> {
        apply_migrations(&self.conn, &self.migrations)
    }

    fn rollback_last(&self) -> DataResult<()> {
        rollback_last_migration(&self.conn, &self.migrations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nest_data::SqlMigration;

    #[test]
    fn apply_and_rollback_migration() {
        let conn = SqliteConnection::open(&crate::config::SqliteConfig::memory()).unwrap();
        let migrations: Vec<Box<dyn Migration>> = vec![Box::new(SqlMigration::new(
            "001_notes",
            "CREATE TABLE notes (id INTEGER PRIMARY KEY, title TEXT NOT NULL);",
            "DROP TABLE notes;",
        ))];
        let runner = SqliteMigrationRunner::new(conn.clone(), migrations);
        assert_eq!(runner.pending().unwrap(), vec!["001_notes"]);
        runner.apply_all().unwrap();
        assert!(runner.pending().unwrap().is_empty());

        conn.with_connection(|db| {
            let exists: i64 = db
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='notes'",
                    [],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(exists, 1);
            Ok(())
        })
        .unwrap();

        runner.rollback_last().unwrap();
        assert_eq!(runner.pending().unwrap(), vec!["001_notes"]);
    }
}
