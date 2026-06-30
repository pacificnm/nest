//! Typed singleton service registry.

use std::any::{Any, TypeId};
use std::collections::HashMap;

use crate::traits::Service;
use nest_error::{NestError, NestResult};

/// A registry of singleton services keyed by concrete type.
///
/// Services are registered explicitly and retrieved by type. v1 supports only
/// singleton lifetimes with `Send + Sync + 'static` types.
///
/// Future versions may add trait-object lookup via
/// `register_service_as::<dyn Trait, Impl>()`.
pub struct ServiceRegistry {
    services: HashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl ServiceRegistry {
    /// Creates an empty service registry.
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    /// Registers a singleton service instance.
    ///
    /// Returns an error if a service of the same type is already registered.
    pub fn register<T: Service>(&mut self, service: T) -> NestResult<()> {
        let type_id = TypeId::of::<T>();
        if self.services.contains_key(&type_id) {
            return Err(NestError::service_already_registered(
                std::any::type_name::<T>(),
            ));
        }
        self.services.insert(type_id, Box::new(service));
        Ok(())
    }

    /// Returns a reference to a registered service by type.
    pub fn get<T: Service>(&self) -> NestResult<&T> {
        let type_id = TypeId::of::<T>();
        let service = self
            .services
            .get(&type_id)
            .ok_or_else(|| NestError::service_not_found(std::any::type_name::<T>()))?;
        service
            .downcast_ref::<T>()
            .ok_or_else(|| NestError::service_not_found(std::any::type_name::<T>()))
    }

    /// Returns a mutable reference to a registered service by type.
    ///
    /// Intended for module configuration before the registry is frozen.
    pub fn get_mut<T: Service>(&mut self) -> NestResult<&mut T> {
        let type_id = TypeId::of::<T>();
        let service = self
            .services
            .get_mut(&type_id)
            .ok_or_else(|| NestError::service_not_found(std::any::type_name::<T>()))?;
        service
            .downcast_mut::<T>()
            .ok_or_else(|| NestError::service_not_found(std::any::type_name::<T>()))
    }

    /// Returns whether a service of the given type is registered.
    pub fn contains<T: Service>(&self) -> bool {
        self.services.contains_key(&TypeId::of::<T>())
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Logger {
        prefix: String,
    }

    #[derive(Debug)]
    struct Counter;

    #[test]
    fn register_and_get_service() {
        let mut registry = ServiceRegistry::new();
        registry
            .register(Logger {
                prefix: "nest".to_string(),
            })
            .unwrap();

        let logger = registry.get::<Logger>().unwrap();
        assert_eq!(logger.prefix, "nest");
    }

    #[test]
    fn duplicate_registration_fails() {
        let mut registry = ServiceRegistry::new();
        registry
            .register(Logger {
                prefix: "a".to_string(),
            })
            .unwrap();

        let err = registry
            .register(Logger {
                prefix: "b".to_string(),
            })
            .unwrap_err();
        assert_eq!(err.kind(), nest_error::NestErrorKind::Service);
        assert_eq!(
            err.code(),
            Some(nest_error::codes::NEST_SERVICE_ALREADY_REGISTERED)
        );
    }

    #[test]
    fn missing_service_fails() {
        let registry = ServiceRegistry::new();
        let err = registry.get::<Logger>().unwrap_err();
        assert_eq!(err.kind(), nest_error::NestErrorKind::Service);
        assert_eq!(err.code(), Some(nest_error::codes::NEST_SERVICE_NOT_FOUND));
    }

    #[test]
    fn contains_returns_correct_state() {
        let mut registry = ServiceRegistry::new();
        assert!(!registry.contains::<Logger>());
        registry
            .register(Logger {
                prefix: "x".to_string(),
            })
            .unwrap();
        assert!(registry.contains::<Logger>());
        assert!(!registry.contains::<Counter>());
    }

    #[test]
    fn get_mut_updates_service() {
        let mut registry = ServiceRegistry::new();
        registry
            .register(Logger {
                prefix: "nest".to_string(),
            })
            .unwrap();

        let logger = registry.get_mut::<Logger>().unwrap();
        logger.prefix = "updated".to_string();
        assert_eq!(registry.get::<Logger>().unwrap().prefix, "updated");
    }
}
