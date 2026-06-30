//! Connection configuration types.

use std::fmt;

/// Named connection identifier (e.g. `primary`, `cache`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ConnectionId(String);

impl ConnectionId {
    /// Primary application database connection.
    pub const PRIMARY: &'static str = "primary";

    /// Creates a connection id from a string.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the id string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for ConnectionId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for ConnectionId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Database engine / provider kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderKind {
    /// SQLite.
    Sqlite,
    /// PostgreSQL.
    Postgres,
    /// MySQL.
    Mysql,
    /// MongoDB.
    Mongo,
    /// Microsoft SQL Server.
    SqlServer,
}

impl ProviderKind {
    /// Returns a short label for logging.
    pub fn label(self) -> &'static str {
        match self {
            Self::Sqlite => "sqlite",
            Self::Postgres => "postgres",
            Self::Mysql => "mysql",
            Self::Mongo => "mongo",
            Self::SqlServer => "sqlserver",
        }
    }
}

/// Connection configuration metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionConfig {
    /// Connection id.
    pub id: ConnectionId,
    /// Provider kind.
    pub provider: ProviderKind,
    /// Datasource string (path, URL, connection string).
    pub datasource: String,
}

impl ConnectionConfig {
    /// Creates a new connection config.
    pub fn new(
        id: impl Into<ConnectionId>,
        provider: ProviderKind,
        datasource: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            provider,
            datasource: datasource.into(),
        }
    }
}
