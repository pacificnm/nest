//! Data layer errors.

use std::error::Error;
use std::fmt;

use crate::codes::{
    NEST_DATA_CONNECTION_ALREADY_REGISTERED, NEST_DATA_CONNECTION_NOT_FOUND, NEST_DATA_FAILED,
    NEST_DATA_MIGRATION_FAILED, NEST_DATA_NO_ACTIVE_CONNECTION,
};
use crate::config::ConnectionId;

/// Result type for data operations.
pub type DataResult<T> = Result<T, DataError>;

/// High-level category for a data error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DataErrorKind {
    /// Entity or row not found.
    NotFound,
    /// Unique constraint or conflict.
    Conflict,
    /// Connection or pool error.
    Connection,
    /// Migration error.
    Migration,
    /// Query execution error.
    Query,
    /// Configuration error.
    Config,
}

/// Structured error for nest-data and provider crates.
#[derive(Debug)]
pub struct DataError {
    kind: DataErrorKind,
    message: String,
    code: Option<String>,
    connection: Option<ConnectionId>,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl DataError {
    /// Creates a new data error.
    pub fn new(kind: DataErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            code: None,
            connection: None,
            source: None,
        }
    }

    /// Creates a not-found error.
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(DataErrorKind::NotFound, message)
    }

    /// Creates a conflict error.
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(DataErrorKind::Conflict, message)
    }

    /// Creates a connection-layer error.
    pub fn connection_error(message: impl Into<String>) -> Self {
        Self::new(DataErrorKind::Connection, message)
    }

    /// Creates a migration error.
    pub fn migration(message: impl Into<String>) -> Self {
        Self::new(DataErrorKind::Migration, message).with_code(NEST_DATA_MIGRATION_FAILED)
    }

    /// Creates a query error.
    pub fn query(message: impl Into<String>) -> Self {
        Self::new(DataErrorKind::Query, message)
    }

    /// Creates a config error.
    pub fn config(message: impl Into<String>) -> Self {
        Self::new(DataErrorKind::Config, message)
    }

    /// Connection not registered.
    pub fn connection_not_found(id: impl AsRef<str>) -> Self {
        Self::connection_error(format!("connection not found: {}", id.as_ref()))
            .with_code(NEST_DATA_CONNECTION_NOT_FOUND)
    }

    /// Connection already registered.
    pub fn connection_already_registered(id: impl AsRef<str>) -> Self {
        Self::connection_error(format!("connection already registered: {}", id.as_ref()))
            .with_code(NEST_DATA_CONNECTION_ALREADY_REGISTERED)
    }

    /// No active connection set.
    pub fn no_active_connection() -> Self {
        Self::connection_error("no active connection").with_code(NEST_DATA_NO_ACTIVE_CONNECTION)
    }

    /// Sets a stable error code.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Sets the connection id context.
    pub fn with_connection(mut self, connection: ConnectionId) -> Self {
        self.connection = Some(connection);
        self
    }

    /// Attaches a source error.
    pub fn with_source(mut self, source: impl Error + Send + Sync + 'static) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    /// Returns the error kind.
    pub fn kind(&self) -> DataErrorKind {
        self.kind
    }

    /// Returns the message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the stable code, if set.
    pub fn code(&self) -> Option<&str> {
        self.code.as_deref()
    }

    /// Returns the connection id context, if set.
    pub fn connection_id(&self) -> Option<&ConnectionId> {
        self.connection.as_ref()
    }
}

impl fmt::Display for DataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for DataError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| source.as_ref() as &(dyn Error + 'static))
    }
}

impl DataError {
    /// Default code when converting to [`nest_error::NestError`].
    pub(crate) fn nest_code(&self) -> &str {
        self.code.as_deref().unwrap_or(NEST_DATA_FAILED)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_not_found_has_code() {
        let err = DataError::connection_not_found("primary");
        assert_eq!(err.code(), Some(NEST_DATA_CONNECTION_NOT_FOUND));
    }
}
