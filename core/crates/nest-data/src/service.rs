//! Runtime data service and connection lifecycle.

use std::sync::{Arc, RwLock};

use crate::config::ConnectionId;
use crate::connection::{ConnectionHealth, ConnectionRegistry, DataConnection};
use crate::error::{DataError, DataResult};

/// Runtime data service: connection registry and active connection state.
///
/// Registered via [`crate::DataModule`]. Uses interior mutability because
/// [`nest_core::AppContext`] exposes services as shared references.
pub struct DataService {
    registry: RwLock<ConnectionRegistry>,
    active: RwLock<Option<ConnectionId>>,
}

impl DataService {
    /// Creates an empty data service.
    pub fn new() -> Self {
        Self {
            registry: RwLock::new(ConnectionRegistry::new()),
            active: RwLock::new(None),
        }
    }

    /// Registers a named connection.
    pub fn register_connection(&self, connection: Arc<dyn DataConnection>) -> DataResult<()> {
        self.registry
            .write()
            .expect("data registry lock")
            .register(connection)
    }

    /// Sets the active connection id.
    pub fn set_active(&self, id: &ConnectionId) -> DataResult<()> {
        if !self
            .registry
            .read()
            .expect("data registry lock")
            .contains(id)
        {
            return Err(DataError::connection_not_found(id.as_str()));
        }
        *self.active.write().expect("active connection lock") = Some(id.clone());
        Ok(())
    }

    /// Returns the active connection id.
    pub fn active_id(&self) -> DataResult<ConnectionId> {
        self.active
            .read()
            .expect("active connection lock")
            .clone()
            .ok_or_else(DataError::no_active_connection)
    }

    /// Returns a registered connection by id.
    pub fn connection(&self, id: &ConnectionId) -> DataResult<Arc<dyn DataConnection>> {
        self.registry.read().expect("data registry lock").get(id)
    }

    /// Returns the active connection.
    pub fn active_connection(&self) -> DataResult<Arc<dyn DataConnection>> {
        let id = self.active_id()?;
        self.connection(&id)
    }

    /// Runs a health check for a connection.
    pub fn health_check(&self, id: &ConnectionId) -> DataResult<ConnectionHealth> {
        self.connection(id)?.health_check()
    }

    /// Lists registered connection ids.
    pub fn list_connections(&self) -> Vec<ConnectionId> {
        self.registry
            .read()
            .expect("data registry lock")
            .ids()
            .map(ConnectionId::new)
            .collect()
    }
}

impl Default for DataService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ConnectionConfig, ProviderKind};
    use crate::connection::ConnectionHealth;

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

    fn test_conn(id: &str) -> Arc<dyn DataConnection> {
        Arc::new(TestConnection {
            config: ConnectionConfig::new(id, ProviderKind::Sqlite, ":memory:"),
        })
    }

    #[test]
    fn register_set_active_and_lookup() {
        let service = DataService::new();
        service.register_connection(test_conn("primary")).unwrap();
        service.set_active(&ConnectionId::new("primary")).unwrap();
        assert_eq!(service.active_id().unwrap().as_str(), "primary");
        assert!(service.active_connection().is_ok());
    }

    #[test]
    fn set_unknown_active_fails() {
        let service = DataService::new();
        let err = service
            .set_active(&ConnectionId::new("missing"))
            .unwrap_err();
        assert_eq!(
            err.code(),
            Some(crate::codes::NEST_DATA_CONNECTION_NOT_FOUND)
        );
    }
}
