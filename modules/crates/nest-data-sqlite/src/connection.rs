//! SQLite connection and transactions.

use std::sync::{Arc, Mutex};
use std::time::Instant;

use nest_data::{
    ConnectionConfig, ConnectionHealth, ConnectionId, DataConnection, DataError, DataResult,
    ProviderKind, Transaction, Transactional,
};
use rusqlite::Connection;

use crate::config::SqliteConfig;
use crate::error::sqlite_result;

struct Inner {
    id: ConnectionId,
    config: ConnectionConfig,
    conn: Mutex<Connection>,
}

/// Shared SQLite connection handle.
#[derive(Clone)]
pub struct SqliteConnection {
    inner: Arc<Inner>,
}

impl SqliteConnection {
    /// Opens a SQLite database from config.
    pub fn open(config: &SqliteConfig) -> DataResult<Self> {
        Self::open_named(ConnectionId::PRIMARY, config)
    }

    /// Opens a named SQLite database from config.
    pub fn open_named(id: impl Into<ConnectionId>, config: &SqliteConfig) -> DataResult<Self> {
        let connection_id = id.into();
        let conn = sqlite_result(Connection::open(&config.path))?;
        for (name, value) in &config.pragmas {
            let sql = format!("PRAGMA {name} = {value}");
            sqlite_result(conn.execute(&sql, []))?;
        }
        let connection_config = ConnectionConfig::new(
            connection_id.clone(),
            ProviderKind::Sqlite,
            config.datasource(),
        );
        Ok(Self {
            inner: Arc::new(Inner {
                id: connection_id,
                config: connection_config,
                conn: Mutex::new(conn),
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

    /// Executes a function with the underlying connection locked.
    pub fn with_connection<F, T>(&self, f: F) -> DataResult<T>
    where
        F: FnOnce(&Connection) -> DataResult<T>,
    {
        let guard = self
            .inner
            .conn
            .lock()
            .map_err(|_| DataError::connection_error("sqlite connection lock poisoned"))?;
        f(&guard)
    }

    /// Returns a clone as [`Arc<dyn DataConnection>`].
    pub fn as_data_connection(self) -> Arc<dyn DataConnection> {
        Arc::new(self)
    }
}

impl DataConnection for SqliteConnection {
    fn id(&self) -> &ConnectionId {
        &self.inner.id
    }

    fn provider(&self) -> ProviderKind {
        ProviderKind::Sqlite
    }

    fn health_check(&self) -> DataResult<ConnectionHealth> {
        let start = Instant::now();
        self.with_connection(|conn| {
            sqlite_result(conn.query_row("SELECT 1", [], |row| row.get::<_, i32>(0)))?;
            Ok(())
        })?;
        Ok(ConnectionHealth::ok(Some(start.elapsed())))
    }
}

struct SqliteTransaction {
    conn: SqliteConnection,
}

impl Transaction for SqliteTransaction {
    fn commit(self: Box<Self>) -> DataResult<()> {
        self.conn.with_connection(|conn| {
            sqlite_result(conn.execute("COMMIT", []))?;
            Ok(())
        })
    }

    fn rollback(self: Box<Self>) -> DataResult<()> {
        self.conn.with_connection(|conn| {
            sqlite_result(conn.execute("ROLLBACK", []))?;
            Ok(())
        })
    }
}

impl Transactional for SqliteConnection {
    fn begin(&self) -> DataResult<Box<dyn Transaction>> {
        self.with_connection(|conn| sqlite_result(conn.execute("BEGIN", [])))?;
        Ok(Box::new(SqliteTransaction {
            conn: self.clone(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_check_in_memory() {
        let conn = SqliteConnection::open(&SqliteConfig::memory()).unwrap();
        let health = conn.health_check().unwrap();
        assert!(health.ok);
    }

    #[test]
    fn transaction_commit_and_rollback() {
        let conn = SqliteConnection::open(&SqliteConfig::memory()).unwrap();
        conn.with_connection(|db| {
            db.execute(
                "CREATE TABLE t (id INTEGER PRIMARY KEY, v TEXT NOT NULL)",
                [],
            )
            .unwrap();
            Ok(())
        })
        .unwrap();

        let tx = conn.begin().unwrap();
        conn.with_connection(|db| {
            db.execute("INSERT INTO t (v) VALUES ('a')", []).unwrap();
            Ok(())
        })
        .unwrap();
        tx.commit().unwrap();

        let count: i64 = conn
            .with_connection(|db| {
                sqlite_result(db.query_row("SELECT COUNT(*) FROM t", [], |r| r.get::<_, i64>(0)))
            })
            .unwrap();
        assert_eq!(count, 1);

        let tx = conn.begin().unwrap();
        conn.with_connection(|db| {
            db.execute("INSERT INTO t (v) VALUES ('b')", []).unwrap();
            Ok(())
        })
        .unwrap();
        tx.rollback().unwrap();

        let count: i64 = conn
            .with_connection(|db| {
                sqlite_result(db.query_row("SELECT COUNT(*) FROM t", [], |r| r.get::<_, i64>(0)))
            })
            .unwrap();
        assert_eq!(count, 1);
    }
}
