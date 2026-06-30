//! Not-empty validator (any non-zero length, no trim).

use crate::context::ValidationContext;
use crate::issue::ValidationIssue;
use crate::validator::{Validator, ValidatorName};

/// Validates that a string is not empty.
pub struct NotEmptyValidator;

impl ValidatorName for NotEmptyValidator {
    const NAME: &'static str = "not_empty";
}

impl Validator<str> for NotEmptyValidator {
    fn validate(&self, value: &str, ctx: &ValidationContext) -> Vec<ValidationIssue> {
        if value.is_empty() {
            vec![ValidationIssue::error(
                ctx.qualify_code("validation.not_empty"),
                "Value must not be empty",
            )]
        } else {
            vec![]
        }
    }
}
