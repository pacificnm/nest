//! Required field validator (non-empty after trim).

use crate::context::ValidationContext;
use crate::issue::ValidationIssue;
use crate::validator::{Validator, ValidatorName};

/// Validates that a string value is present (not empty after trim).
pub struct RequiredValidator;

impl ValidatorName for RequiredValidator {
    const NAME: &'static str = "required";
}

impl Validator<str> for RequiredValidator {
    fn validate(&self, value: &str, ctx: &ValidationContext) -> Vec<ValidationIssue> {
        if value.trim().is_empty() {
            vec![ValidationIssue::error(
                ctx.qualify_code("validation.required"),
                "This field is required",
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
    fn rejects_blank() {
        let issues = RequiredValidator.validate("   ", &ValidationContext::new());
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn accepts_value() {
        let issues = RequiredValidator.validate("x", &ValidationContext::new());
        assert!(issues.is_empty());
    }
}
