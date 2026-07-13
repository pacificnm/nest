//! PostgreSQL connection pool and transactions.

use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use nest_data::{
    AsyncTransaction, AsyncTransactional, ConnectionConfig, ConnectionHealth, ConnectionId,
    DataConnection, DataResult, ProviderKind,
};
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Postgres};

use crate::config::PostgresConfig;
use crate::error::sqlx_result;
use crate::runtime::block_on;

struct Inner {
    id: ConnectionId,
    config: ConnectionConfig,
    pool: PgPool,
}

/// Shared PostgreSQL pool handle.
#[derive(Clone)]
pub struct PostgresConnection {
    inner: Arc<Inner>,
}

impl PostgresConnection {
    /// Connects to PostgreSQL using the given config.
    pub async fn connect(config: &PostgresConfig) -> DataResult<Self> {
        Self::connect_named(ConnectionId::PRIMARY, config).await
    }

    /// Connects to a named PostgreSQL database.
    pub async fn connect_named(
        id: impl Into<ConnectionId>,
        config: &PostgresConfig,
    ) -> DataResult<Self> {
        let connection_id = id.into();
        let mut attempt: u32 = 0;
        let mut backoff_ms = config.connect_backoff_ms;
        let pool = loop {
            attempt += 1;
            match PgPoolOptions::new()
                .max_connections(config.max_connections)
                .connect(&config.url)
                .await
            {
                Ok(pool) => break pool,
                Err(err) if attempt < config.connect_retries => {
                    tracing::warn!(
                        attempt,
                        max_attempts = config.connect_retries,
                        error = %err,
                        "postgres connection attempt failed, retrying"
                    );
                    tokio::time::sleep(std::time::Duration::from_millis(backoff_ms)).await;
                    backoff_ms = (backoff_ms * 2).min(config.connect_backoff_max_ms);
                }
                Err(err) => return Err(crate::error::map_sqlx_error(err)),
            }
        };
        let connection_config = ConnectionConfig::new(
            connection_id.clone(),
            ProviderKind::Postgres,
            config.datasource(),
        );
        Ok(Self {
            inner: Arc::new(Inner {
                id: connection_id,
                config: connection_config,
                pool,
            }),
        })
    }

    /// Returns the connection id.
    pub fn connection_id(&self) -> &ConnectionId {
        &self.inner.id
    }

    /// Returns connection metadata.
    pub fn connection_config(&self) -> &ConnectionConfig {
        &self.inner.config
    }

    /// Returns the underlying connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.inner.pool
    }

    /// Async health probe (`SELECT 1`). Prefer this over [`DataConnection::health_check`]
    /// when already on a Tokio runtime (the sync health check uses `block_on`).
    pub async fn ping(&self) -> DataResult<()> {
        sqlx_result(sqlx::query("SELECT 1").execute(self.pool()).await)?;
        Ok(())
    }

    /// Returns a clone as [`Arc<dyn DataConnection>`].
    pub fn as_data_connection(self) -> Arc<dyn DataConnection> {
        Arc::new(self)
    }
}

impl DataConnection for PostgresConnection {
    fn id(&self) -> &ConnectionId {
        &self.inner.id
    }

    fn provider(&self) -> ProviderKind {
        ProviderKind::Postgres
    }

    fn health_check(&self) -> DataResult<ConnectionHealth> {
        let pool = self.inner.pool.clone();
        block_on(async move {
            let start = Instant::now();
            sqlx_result(sqlx::query("SELECT 1").execute(&pool).await)?;
            Ok(ConnectionHealth::ok(Some(start.elapsed())))
        })
    }
}

struct PostgresTransaction {
    conn: sqlx::pool::PoolConnection<Postgres>,
}

#[async_trait]
impl AsyncTransaction for PostgresTransaction {
    async fn commit(mut self: Box<Self>) -> DataResult<()> {
        sqlx_result(sqlx::query("COMMIT").execute(&mut *self.conn).await)?;
        Ok(())
    }

    async fn rollback(mut self: Box<Self>) -> DataResult<()> {
        sqlx_result(sqlx::query("ROLLBACK").execute(&mut *self.conn).await)?;
        Ok(())
    }
}

#[async_trait]
impl AsyncTransactional for PostgresConnection {
    async fn begin(&self) -> DataResult<Box<dyn AsyncTransaction>> {
        let mut conn = sqlx_result(self.inner.pool.acquire().await)?;
        sqlx_result(sqlx::query("BEGIN").execute(&mut *conn).await)?;
        Ok(Box::new(PostgresTransaction { conn }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "requires DATABASE_URL and PostgreSQL"]
    async fn health_check_live() {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
        let conn = PostgresConnection::connect(&PostgresConfig::new(url))
            .await
            .unwrap();
        let health = conn.health_check().unwrap();
        assert!(health.ok);
    }

    #[tokio::test]
    async fn connect_retries_before_failing() {
        // Port 1 is a reserved/unlikely-to-be-open port; connection should fail fast each attempt.
        let config = PostgresConfig::new("postgresql://user:pass@127.0.0.1:1/db")
            .with_connect_retries(3)
            .with_connect_backoff(10, 50);
        let start = std::time::Instant::now();
        let result = PostgresConnection::connect(&config).await;
        assert!(result.is_err());
        // 3 attempts with backoff 10ms then 20ms between them = at least 30ms elapsed.
        assert!(start.elapsed() >= std::time::Duration::from_millis(30));
    }
}
