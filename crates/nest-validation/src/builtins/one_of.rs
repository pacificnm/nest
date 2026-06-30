//! One-of allowed values validator.

use crate::context::ValidationContext;
use crate::issue::ValidationIssue;
use crate::validator::NamedValidator;

/// Validates that a string is one of the allowed values.
pub struct OneOfValidator {
    name: &'static str,
    allowed: Vec<String>,
}

impl OneOfValidator {
    /// Creates a one-of validator with the default registry name.
    pub fn new(allowed: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::named("one_of", allowed)
    }

    /// Creates a one-of validator with a custom registry name.
    pub fn named(
        name: &'static str,
        allowed: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            name,
            allowed: allowed.into_iter().map(Into::into).collect(),
        }
    }
}

impl NamedValidator for OneOfValidator {
    fn name(&self) -> &'static str {
        self.name
    }

    fn validate_str(&self, value: &str, ctx: &ValidationContext) -> Vec<ValidationIssue> {
        if self.allowed.iter().any(|item| item == value) {
            vec![]
        } else {
            vec![ValidationIssue::error(
                ctx.qualify_code("validation.one_of"),
                "Value is not an allowed option",
            )]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allowed_value_passes() {
        let validator = OneOfValidator::new(["a", "b"]);
        assert!(validator
            .validate_str("a", &ValidationContext::new())
            .is_empty());
    }

    #[test]
    fn disallowed_value_fails() {
        let validator = OneOfValidator::new(["a", "b"]);
        assert!(!validator
            .validate_str("c", &ValidationContext::new())
            .is_empty());
    }
}
