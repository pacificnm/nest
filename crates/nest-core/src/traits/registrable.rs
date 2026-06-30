//! Shared registration metadata for extension-point types.

use std::any::TypeId;

/// Metadata collected when registering extension-point types (panels, commands, etc.).
#[derive(Debug, Clone)]
pub struct RegistrationInfo {
    /// Stable identifier for the registered item.
    pub id: String,
    /// The concrete Rust type that was registered.
    pub type_id: TypeId,
    /// Human-readable type name for debugging and introspection.
    pub type_name: String,
}

impl RegistrationInfo {
    /// Creates registration metadata for a type with the given id.
    pub fn new<T: 'static>(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            type_id: TypeId::of::<T>(),
            type_name: std::any::type_name::<T>().to_string(),
        }
    }
}
