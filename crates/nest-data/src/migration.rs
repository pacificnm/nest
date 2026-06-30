//! Migration contracts.

use crate::error::DataResult;

/// A versioned schema migration.
pub trait Migration: Send + Sync {
    /// Stable migration id (e.g. `001_create_notes`).
    fn id(&self) -> &str;

    /// SQL to apply the migration.
    fn up_sql(&self) -> &str;

    /// SQL to roll back the migration.
    fn down_sql(&self) -> &str;
}

/// Runs pending migrations for a provider connection.
pub trait MigrationRunner: Send + Sync {
    /// Returns ids of migrations not yet applied.
    fn pending(&self) -> DataResult<Vec<String>>;

    /// Applies all pending migrations in order.
    fn apply_all(&self) -> DataResult<()>;

    /// Rolls back the most recently applied migration.
    fn rollback_last(&self) -> DataResult<()>;
}

/// Simple in-memory migration for tests and examples.
#[derive(Debug, Clone)]
pub struct SqlMigration {
    id: String,
    up: String,
    down: String,
}

impl SqlMigration {
    /// Creates a migration from id and SQL strings.
    pub fn new(
        id: impl Into<String>,
        up_sql: impl Into<String>,
        down_sql: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            up: up_sql.into(),
            down: down_sql.into(),
        }
    }
}

impl Migration for SqlMigration {
    fn id(&self) -> &str {
        &self.id
    }

    fn up_sql(&self) -> &str {
        &self.up
    }

    fn down_sql(&self) -> &str {
        &self.down
    }
}
