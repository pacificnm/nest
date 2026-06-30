//! Minimum and maximum string length validators.

use crate::context::ValidationContext;
use crate::issue::ValidationIssue;
use crate::validator::NamedValidator;

/// Validates minimum string length (in Unicode scalar values).
pub struct MinLengthValidator {
    min: usize,
    name: &'static str,
}

impl MinLengthValidator {
    /// Creates a validator with a custom minimum length.
    pub fn new(min: usize) -> Self {
        Self {
            min,
            name: "min_length",
        }
    }

    /// Creates a validator with an explicit registry name.
    pub fn named(name: &'static str, min: usize) -> Self {
        Self { min, name }
    }
}

impl Default for MinLengthValidator {
    fn default() -> Self {
        Self::new(1)
    }
}

impl NamedValidator for MinLengthValidator {
    fn name(&self) -> &'static str {
        self.name
    }

    fn validate_str(&self, value: &str, ctx: &ValidationContext) -> Vec<ValidationIssue> {
        if value.chars().count() < self.min {
            vec![ValidationIssue::error(
                ctx.qualify_code("validation.min_length"),
                format!("Must be at least {} characters", self.min),
            )]
        } else {
            vec![]
        }
    }
}

/// Validates maximum string length (in Unicode scalar values).
pub struct MaxLengthValidator {
    max: usize,
    name: &'static str,
}

impl MaxLengthValidator {
    /// Creates a validator with a custom maximum length.
    pub fn new(max: usize) -> Self {
        Self {
            max,
            name: "max_length",
        }
    }

    /// Creates a validator with an explicit registry name.
    pub fn named(name: &'static str, max: usize) -> Self {
        Self { max, name }
    }
}

impl Default for MaxLengthValidator {
    fn default() -> Self {
        Self::new(255)
    }
}

impl NamedValidator for MaxLengthValidator {
    fn name(&self) -> &'static str {
        self.name
    }

    fn validate_str(&self, value: &str, ctx: &ValidationContext) -> Vec<ValidationIssue> {
        if value.chars().count() > self.max {
            vec![ValidationIssue::error(
                ctx.qualify_code("validation.max_length"),
                format!("Must be at most {} characters", self.max),
            )]
        } else {
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn min_length_enforced() {
        let validator = MinLengthValidator::new(3);
        assert!(!validator
            .validate_str("ab", &ValidationContext::new())
            .is_empty());
        assert!(validator
            .validate_str("abc", &ValidationContext::new())
            .is_empty());
    }

    #[test]
    fn max_length_enforced() {
        let validator = MaxLengthValidator::new(3);
        assert!(!validator
            .validate_str("abcd", &ValidationContext::new())
            .is_empty());
    }
}
