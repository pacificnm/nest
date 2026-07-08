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
        let pool = sqlx_result(
            PgPoolOptions::new()
                .max_connections(config.max_connections)
                .connect(&config.url)
                .await,
        )?;
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
}
