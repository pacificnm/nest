//! Object-level validation trait.

use crate::context::ValidationContext;
use crate::error::ValidationResult;

/// Validates a value and returns all collected issues.
pub trait Validate {
    /// Validates `self` using the given context.
    fn validate(&self, ctx: &ValidationContext) -> ValidationResult;
}

/// Validates a value with a default context.
pub fn validate<T: Validate>(value: &T) -> ValidationResult {
    validate_with_context(value, &ValidationContext::new())
}

/// Validates a value with an explicit context.
pub fn validate_with_context<T: Validate>(value: &T, ctx: &ValidationContext) -> ValidationResult {
    value.validate(ctx)
}
