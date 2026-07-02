//! Built-in validators.

mod email;
mod length;
mod not_empty;
mod one_of;
mod range;
#[cfg(feature = "regex")]
mod regex_validator;
mod required;
mod url;

pub use email::EmailValidator;
pub use length::{MaxLengthValidator, MinLengthValidator};
pub use not_empty::NotEmptyValidator;
pub use one_of::OneOfValidator;
pub use range::RangeValidator;
#[cfg(feature = "regex")]
pub use regex_validator::RegexValidator;
pub use required::RequiredValidator;
pub use url::UrlValidator;

use crate::context::ValidationContext;
use crate::error::{ValidationError, ValidationResult};
use crate::issue::ValidationIssue;

/// Registers all built-in string validators with default configuration.
pub fn register_defaults(
    registry: &mut crate::registry::ValidatorRegistry,
) -> nest_error::NestResult<()> {
    registry.register(RequiredValidator)?;
    registry.register(NotEmptyValidator)?;
    registry.register(EmailValidator)?;
    registry.register(UrlValidator)?;
    registry.register(MinLengthValidator::default())?;
    registry.register(MaxLengthValidator::default())?;
    Ok(())
}

/// Merges issue vectors into a single validation result.
pub fn merge_issues(groups: impl IntoIterator<Item = Vec<ValidationIssue>>) -> ValidationResult {
    let issues: Vec<ValidationIssue> = groups.into_iter().flatten().collect();
    ValidationError::from_issues(issues)
}

/// Merges issue vectors, stopping early when configured and a blocking issue appears.
pub fn merge_issues_with_context(
    ctx: &ValidationContext,
    groups: impl IntoIterator<Item = Vec<ValidationIssue>>,
) -> ValidationResult {
    let mut issues = Vec::new();
    for group in groups {
        for issue in group {
            let blocking = issue.is_blocking();
            issues.push(issue);
            if ctx.stop_on_first_error() && blocking {
                return ValidationError::from_issues(issues);
            }
        }
    }
    ValidationError::from_issues(issues)
}
