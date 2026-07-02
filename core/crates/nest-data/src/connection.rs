//! Connection contracts and registry.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::config::{ConnectionId, ProviderKind};
use crate::error::{DataError, DataResult};

/// Result of a connection health check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionHealth {
    /// Whether the connection is healthy.
    pub ok: bool,
    /// Round-trip latency, if measured.
    pub latency: Option<Duration>,
    /// Optional detail message.
    pub message: Option<String>,
}

impl ConnectionHealth {
    /// Creates a healthy result.
    pub fn ok(latency: Option<Duration>) -> Self {
        Self {
            ok: true,
            latency,
            message: None,
        }
    }

    /// Creates an unhealthy result.
    pub fn failed(message: impl Into<String>) -> Self {
        Self {
            ok: false,
            latency: None,
            message: Some(message.into()),
        }
    }
}

/// A registered database connection.
pub trait DataConnection: Send + Sync {
    /// Returns the connection id.
    fn id(&self) -> &ConnectionId;

    /// Returns the provider kind.
    fn provider(&self) -> ProviderKind;

    /// Checks whether the connection is usable.
    fn health_check(&self) -> DataResult<ConnectionHealth>;
}

/// In-memory registry of named connections.
#[derive(Default)]
pub struct ConnectionRegistry {
    connections: HashMap<String, Arc<dyn DataConnection>>,
}

impl ConnectionRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a connection by id.
    pub fn register(&mut self, connection: Arc<dyn DataConnection>) -> DataResult<()> {
        let id = connection.id().as_str().to_string();
        if self.connections.contains_key(&id) {
            return Err(DataError::connection_already_registered(&id));
        }
        self.connections.insert(id, connection);
        Ok(())
    }

    /// Returns whether a connection id is registered.
    pub fn contains(&self, id: &ConnectionId) -> bool {
        self.connections.contains_key(id.as_str())
    }

    /// Returns a connection by id.
    pub fn get(&self, id: &ConnectionId) -> DataResult<Arc<dyn DataConnection>> {
        self.connections
            .get(id.as_str())
            .cloned()
            .ok_or_else(|| DataError::connection_not_found(id.as_str()))
    }

    /// Lists registered connection ids.
    pub fn ids(&self) -> impl Iterator<Item = &str> {
        self.connections.keys().map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConnectionConfig;

    struct TestConnection {
        config: ConnectionConfig,
    }

    impl DataConnection for TestConnection {
        fn id(&self) -> &ConnectionId {
            &self.config.id
        }

        fn provider(&self) -> ProviderKind {
            self.config.provider
        }

        fn health_check(&self) -> DataResult<ConnectionHealth> {
            Ok(ConnectionHealth::ok(None))
        }
    }

    #[test]
    fn register_and_get_connection() {
        let mut registry = ConnectionRegistry::new();
        let config = ConnectionConfig::new(ConnectionId::PRIMARY, ProviderKind::Sqlite, ":memory:");
        let conn = Arc::new(TestConnection { config }) as Arc<dyn DataConnection>;
        registry.register(conn).unwrap();
        assert!(registry.contains(&ConnectionId::new(ConnectionId::PRIMARY)));
        assert!(registry
            .get(&ConnectionId::new(ConnectionId::PRIMARY))
            .is_ok());
    }

    #[test]
    fn duplicate_registration_fails() {
        let mut registry = ConnectionRegistry::new();
        let config = ConnectionConfig::new(ConnectionId::PRIMARY, ProviderKind::Sqlite, ":memory:");
        registry
            .register(Arc::new(TestConnection {
                config: config.clone(),
            }))
            .unwrap();
        let err = registry
            .register(Arc::new(TestConnection { config }))
            .unwrap_err();
        assert_eq!(
            err.code(),
            Some(crate::codes::NEST_DATA_CONNECTION_ALREADY_REGISTERED)
        );
    }
}
