//! Service marker trait for types registered in the service registry.

/// Marker trait for types that can be registered as singleton services.
///
/// v1 services must be `Send + Sync + 'static` and are registered explicitly.
/// No constructor injection, factories, or scoped lifetimes are supported yet.
pub trait Service: Send + Sync + 'static {}

impl<T: Send + Sync + 'static> Service for T {}
