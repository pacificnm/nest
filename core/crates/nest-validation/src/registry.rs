//! Validator registry for named field validators.

use std::collections::HashMap;

use nest_error::{NestError, NestResult};

use crate::codes::{NEST_VALIDATOR_ALREADY_REGISTERED, NEST_VALIDATOR_NOT_FOUND};
use crate::context::ValidationContext;
use crate::issue::ValidationIssue;
use crate::validator::NamedValidator;

/// Registry of named string-field validators.
///
/// Registered as a singleton via [`crate::ValidationModule`]. Extended during
/// module configuration through [`nest_core::AppBuilder::service_mut`].
pub struct ValidatorRegistry {
    validators: HashMap<&'static str, Box<dyn NamedValidator>>,
}

impl ValidatorRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self {
            validators: HashMap::new(),
        }
    }

    /// Registers a named validator.
    pub fn register<V: NamedValidator + 'static>(&mut self, validator: V) -> NestResult<()> {
        let name = validator.name();
        if self.validators.contains_key(name) {
            return Err(
                NestError::validation(format!("validator already registered: {name}"))
                    .with_code(NEST_VALIDATOR_ALREADY_REGISTERED),
            );
        }
        self.validators.insert(name, Box::new(validator));
        Ok(())
    }

    /// Returns whether a validator is registered.
    pub fn contains(&self, name: &str) -> bool {
        self.validators.contains_key(name)
    }

    /// Returns registered validator names.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.validators.keys().copied()
    }

    /// Runs a named validator against a string value.
    pub fn validate_named(
        &self,
        name: &str,
        value: &str,
        ctx: &ValidationContext,
    ) -> NestResult<Vec<ValidationIssue>> {
        let validator = self.validators.get(name).ok_or_else(|| {
            NestError::validation(format!("validator not found: {name}"))
                .with_code(NEST_VALIDATOR_NOT_FOUND)
        })?;
        Ok(validator.validate_str(value, ctx))
    }
}

impl Default for ValidatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtins::EmailValidator;

    #[test]
    fn register_and_validate_named() {
        let mut registry = ValidatorRegistry::new();
        registry.register(EmailValidator).unwrap();
        let ctx = ValidationContext::new();
        let issues = registry.validate_named("email", "bad", &ctx).unwrap();
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn duplicate_registration_fails() {
        let mut registry = ValidatorRegistry::new();
        registry.register(EmailValidator).unwrap();
        let err = registry.register(EmailValidator).unwrap_err();
        assert_eq!(err.code(), Some(NEST_VALIDATOR_ALREADY_REGISTERED));
    }
}
