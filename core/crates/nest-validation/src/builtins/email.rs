//! Basic email format validator.

use crate::context::ValidationContext;
use crate::issue::ValidationIssue;
use crate::validator::{Validator, ValidatorName};

/// Validates a basic email shape (contains `@` with non-empty local and domain).
pub struct EmailValidator;

impl ValidatorName for EmailValidator {
    const NAME: &'static str = "email";
}

impl Validator<str> for EmailValidator {
    fn validate(&self, value: &str, ctx: &ValidationContext) -> Vec<ValidationIssue> {
        let valid = value.split_once('@').is_some_and(|(local, domain)| {
            !local.is_empty() && !domain.is_empty() && domain.contains('.')
        });
        if valid {
            vec![]
        } else {
            vec![ValidationIssue::error(
                ctx.qualify_code("validation.email"),
                "Invalid email address",
            )]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_email() {
        assert!(EmailValidator
            .validate("user@example.com", &ValidationContext::new())
            .is_empty());
    }

    #[test]
    fn rejects_invalid_email() {
        assert!(!EmailValidator
            .validate("not-an-email", &ValidationContext::new())
            .is_empty());
    }
}
