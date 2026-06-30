//! Field-level validator trait.

use crate::context::ValidationContext;
use crate::issue::ValidationIssue;

/// Validates a single typed value and returns zero or more issues.
pub trait Validator<T: ?Sized>: Send + Sync + 'static {
    /// Validates `value` and returns all issues found.
    fn validate(&self, value: &T, ctx: &ValidationContext) -> Vec<ValidationIssue>;
}

/// Stable name for a validator registered by type.
pub trait ValidatorName {
    /// Registry lookup name (e.g. `email`, `required`).
    const NAME: &'static str;
}

/// Type-erased string-field validator for schema-driven hosts.
pub trait NamedValidator: Send + Sync {
    /// Registry lookup name.
    fn name(&self) -> &'static str;

    /// Validates a string field value.
    fn validate_str(&self, value: &str, ctx: &ValidationContext) -> Vec<ValidationIssue>;
}

impl<V> NamedValidator for V
where
    V: Validator<str> + ValidatorName,
{
    fn name(&self) -> &'static str {
        V::NAME
    }

    fn validate_str(&self, value: &str, ctx: &ValidationContext) -> Vec<ValidationIssue> {
        V::validate(self, value, ctx)
    }
}
