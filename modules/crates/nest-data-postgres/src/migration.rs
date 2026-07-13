//! PostgreSQL migration runner.

use nest_data::{DataError, DataResult, Migration, MigrationRunner};
use sqlx::AssertSqlSafe;
use sqlx::{PgPool, Row};

use crate::connection::PostgresConnection;
use crate::error::sqlx_result;

const CREATE_MIGRATIONS_TABLE: &str = "CREATE TABLE IF NOT EXISTS _nest_migrations (
    id TEXT PRIMARY KEY NOT NULL,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
)";

const SELECT_APPLIED_MIGRATIONS: &str = "SELECT id FROM _nest_migrations ORDER BY applied_at ASC";

const INSERT_MIGRATION: &str = "INSERT INTO _nest_migrations (id) VALUES ($1)";

const DELETE_MIGRATION: &str = "DELETE FROM _nest_migrations WHERE id = $1";

async fn ensure_table(pool: &PgPool) -> DataResult<()> {
    sqlx_result(sqlx::query(CREATE_MIGRATIONS_TABLE).execute(pool).await)?;
    Ok(())
}

async fn applied_ids(pool: &PgPool) -> DataResult<Vec<String>> {
    ensure_table(pool).await?;
    let rows = sqlx_result(sqlx::query(SELECT_APPLIED_MIGRATIONS).fetch_all(pool).await)?;
    Ok(rows.into_iter().map(|row| row.get("id")).collect())
}

async fn pending_ids(pool: &PgPool, migrations: &[Box<dyn Migration>]) -> DataResult<Vec<String>> {
    let applied: std::collections::HashSet<String> = applied_ids(pool).await?.into_iter().collect();
    Ok(migrations
        .iter()
        .map(|m| m.id().to_string())
        .filter(|id| !applied.contains(id))
        .collect())
}

/// Applies pending migrations to a PostgreSQL pool.
pub async fn apply_migrations(pool: &PgPool, migrations: &[Box<dyn Migration>]) -> DataResult<()> {
    let pending = pending_ids(pool, migrations).await?;
    for migration in migrations {
        if !pending.iter().any(|id| id == migration.id()) {
            continue;
        }
        let up_sql = migration.up_sql().to_string();
        sqlx_result(sqlx::raw_sql(AssertSqlSafe(up_sql)).execute(pool).await).map_err(|error| {
            DataError::migration(format!(
                "failed to apply migration `{}`: {error}",
                migration.id()
            ))
        })?;
        sqlx_result(
            sqlx::query(INSERT_MIGRATION)
                .bind(migration.id())
                .execute(pool)
                .await,
        )
        .map_err(|error| {
            DataError::migration(format!(
                "failed to record migration `{}`: {error}",
                migration.id()
            ))
        })?;
    }
    Ok(())
}

/// Rolls back the most recently applied migration.
pub async fn rollback_last_migration(
    pool: &PgPool,
    migrations: &[Box<dyn Migration>],
) -> DataResult<()> {
    let applied = applied_ids(pool).await?;
    let last_id = applied
        .last()
        .ok_or_else(|| DataError::migration("no migrations to roll back"))?;

    let migration = migrations
        .iter()
        .find(|m| m.id() == last_id)
        .ok_or_else(|| DataError::migration(format!("migration `{last_id}` is not registered")))?;

    let down_sql = migration.down_sql().to_string();
    sqlx_result(sqlx::raw_sql(AssertSqlSafe(down_sql)).execute(pool).await)?;
    sqlx_result(
        sqlx::query(DELETE_MIGRATION)
            .bind(migration.id())
            .execute(pool)
            .await,
    )?;
    Ok(())
}

/// Runs migrations against a PostgreSQL connection pool.
pub struct PostgresMigrationRunner {
    conn: PostgresConnection,
    migrations: Vec<Box<dyn Migration>>,
}

impl PostgresMigrationRunner {
    /// Creates a runner with the given migrations.
    pub fn new(conn: PostgresConnection, migrations: Vec<Box<dyn Migration>>) -> Self {
        Self { conn, migrations }
    }

    /// Returns the underlying connection.
    pub fn connection(&self) -> &PostgresConnection {
        &self.conn
    }
}

impl MigrationRunner for PostgresMigrationRunner {
    fn pending(&self) -> DataResult<Vec<String>> {
        crate::runtime::block_on(pending_ids(self.conn.pool(), &self.migrations))
    }

    fn apply_all(&self) -> DataResult<()> {
        crate::runtime::block_on(apply_migrations(self.conn.pool(), &self.migrations))
    }

    fn rollback_last(&self) -> DataResult<()> {
        crate::runtime::block_on(rollback_last_migration(self.conn.pool(), &self.migrations))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nest_data::SqlMigration;

    #[tokio::test(flavor = "multi_thread")]
    async fn apply_and_rollback_migration() {
        let db = crate::test_support::start_postgres().await;
        let conn = PostgresConnection::connect(&crate::config::PostgresConfig::new(db.url))
            .await
            .unwrap();
        let migrations: Vec<Box<dyn Migration>> = vec![Box::new(SqlMigration::new(
            "001_notes",
            "CREATE TABLE notes (id SERIAL PRIMARY KEY, title TEXT NOT NULL);",
            "DROP TABLE notes;",
        ))];
        let runner = PostgresMigrationRunner::new(conn.clone(), migrations);
        assert_eq!(runner.pending().unwrap(), vec!["001_notes"]);
        runner.apply_all().unwrap();
        assert!(runner.pending().unwrap().is_empty());

        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS (
                SELECT 1 FROM information_schema.tables
                WHERE table_schema = 'public' AND table_name = 'notes'
            )",
        )
        .fetch_one(conn.pool())
        .await
        .unwrap();
        assert!(exists);

        runner.rollback_last().unwrap();
        assert_eq!(runner.pending().unwrap(), vec!["001_notes"]);
    }
}
