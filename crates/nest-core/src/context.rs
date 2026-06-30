//! Application context for runtime service lookup.

use crate::registry::ServiceRegistry;
use crate::traits::Service;
use nest_error::NestResult;

/// Runtime context providing access to registered services.
///
/// Created when [`crate::builder::BuiltApp`] is built and shared via `Arc` so
/// UI closures and modules can hold a handle. The service registry is frozen
/// after build; services cannot be registered post-build.
pub struct AppContext {
    services: ServiceRegistry,
}

impl AppContext {
    /// Creates a new context from a built service registry.
    pub(crate) fn new(services: ServiceRegistry) -> Self {
        Self { services }
    }

    /// Returns a reference to a registered singleton service.
    pub fn service<T: Service>(&self) -> NestResult<&T> {
        self.services.get::<T>()
    }

    /// Returns whether a service of the given type is registered.
    pub fn has_service<T: Service>(&self) -> bool {
        self.services.contains::<T>()
    }
}
