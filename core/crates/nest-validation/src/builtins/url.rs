//! Basic URL format validator.

use crate::context::ValidationContext;
use crate::issue::ValidationIssue;
use crate::validator::{Validator, ValidatorName};

/// Validates http/https URL prefixes.
pub struct UrlValidator;

impl ValidatorName for UrlValidator {
    const NAME: &'static str = "url";
}

impl Validator<str> for UrlValidator {
    fn validate(&self, value: &str, ctx: &ValidationContext) -> Vec<ValidationIssue> {
        let valid = value.starts_with("http://") || value.starts_with("https://");
        if valid {
            vec![]
        } else {
            vec![
                ValidationIssue::error(ctx.qualify_code("validation.url"), "Invalid URL")
                    .with_help("URL must start with http:// or https://"),
            ]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_https() {
        assert!(UrlValidator
            .validate("https://example.com", &ValidationContext::new())
            .is_empty());
    }

    #[test]
    fn rejects_missing_scheme() {
        assert!(!UrlValidator
            .validate("example.com", &ValidationContext::new())
            .is_empty());
    }
}
