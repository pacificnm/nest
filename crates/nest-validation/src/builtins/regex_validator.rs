//! Regex pattern validator.

use crate::context::ValidationContext;
use crate::issue::ValidationIssue;
use crate::validator::NamedValidator;

/// Validates a string against a regex pattern.
#[cfg(feature = "regex")]
pub struct RegexValidator {
    name: &'static str,
    pattern: regex::Regex,
}

#[cfg(feature = "regex")]
impl RegexValidator {
    /// Creates a regex validator with the default registry name `regex`.
    pub fn new(pattern: &str) -> nest_error::NestResult<Self> {
        Self::named("regex", pattern)
    }

    /// Creates a regex validator with a custom registry name.
    pub fn named(name: &'static str, pattern: &str) -> nest_error::NestResult<Self> {
        let pattern = regex::Regex::new(pattern).map_err(|error| {
            nest_error::NestError::validation(format!("invalid regex pattern: {error}"))
        })?;
        Ok(Self { name, pattern })
    }
}

#[cfg(feature = "regex")]
impl NamedValidator for RegexValidator {
    fn name(&self) -> &'static str {
        self.name
    }

    fn validate_str(&self, value: &str, ctx: &ValidationContext) -> Vec<ValidationIssue> {
        if self.pattern.is_match(value) {
            vec![]
        } else {
            vec![ValidationIssue::error(
                ctx.qualify_code("validation.regex"),
                "Value does not match the required pattern",
            )]
        }
    }
}

#[cfg(all(test, feature = "regex"))]
mod tests {
    use super::*;

    #[test]
    fn regex_matches() {
        let validator = RegexValidator::new(r"^\d+$").unwrap();
        assert!(validator
            .validate_str("123", &ValidationContext::new())
            .is_empty());
        assert!(!validator
            .validate_str("abc", &ValidationContext::new())
            .is_empty());
    }
}
