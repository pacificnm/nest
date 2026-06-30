//! UI-agnostic validation for the Nest framework.
//!
//! nest-validation provides structured validation issues, field-level validators,
//! object-level [`Validate`] implementations, and a [`ValidatorRegistry`]
//! registered via [`ValidationModule`].
//!
//! Hosts (forms, API, CLI, imports) decide how to present issues; this crate
//! owns validation logic only.

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

pub mod builtins;
pub mod codes;
mod context;
mod error;
mod issue;
mod module;
mod prelude;
mod registry;
mod validate;
mod validator;

pub use context::ValidationContext;
pub use error::{ValidationError, ValidationResult};
pub use issue::{FieldPath, Severity, ValidationIssue};
pub use module::{ValidationModule, VALIDATION_MODULE_ID};
pub use registry::ValidatorRegistry;
pub use validate::{validate, validate_with_context, Validate};
pub use validator::{NamedValidator, Validator, ValidatorName};

pub use nest_core::{Module, ModuleId};
pub use nest_error::{NestError, NestResult};

use crate::codes::NEST_VALIDATION_FAILED;

impl From<ValidationError> for NestError {
    fn from(error: ValidationError) -> Self {
        let blocking = error.blocking_issues().count();
        let help = if blocking > 1 {
            Some(format!("{blocking} validation errors; see issue details."))
        } else {
            error
                .blocking_issues()
                .next()
                .and_then(|issue| issue.help.clone())
        };

        let mut nest_error = NestError::validation(error.summary_message())
            .with_code(NEST_VALIDATION_FAILED)
            .with_module("nest-validation");

        if let Some(help) = help {
            nest_error = nest_error.with_help(help);
        }

        nest_error.with_source(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builtins::{EmailValidator, RequiredValidator};
    use crate::builtins::{merge_issues, RangeValidator};

    struct Project {
        name: String,
        start: i32,
        end: i32,
    }

    impl Validate for Project {
        fn validate(&self, ctx: &ValidationContext) -> ValidationResult {
            let mut issues = Vec::new();

            if self.name.trim().is_empty() {
                issues.push(ValidationIssue::field_error(
                    "name",
                    ctx.qualify_code("validation.required"),
                    "Project name is required",
                ));
            }

            if self.start > self.end {
                issues.push(ValidationIssue::field_error(
                    "end_date",
                    ctx.qualify_code("validation.date_range"),
                    "End date must be after start date",
                ));
            }

            ValidationError::from_issues(issues)
        }
    }

    #[test]
    fn object_validation_collects_multiple_issues() {
        let project = Project {
            name: "   ".to_string(),
            start: 10,
            end: 1,
        };
        let err = validate(&project).unwrap_err();
        assert_eq!(err.issues().len(), 2);
    }

    #[test]
    fn warnings_do_not_block_by_default() {
        let issues = vec![
            ValidationIssue::field_warning("name", "validation.weak", "Name is short"),
        ];
        assert!(ValidationError::from_issues(issues).is_ok());
    }

    #[test]
    fn merge_issues_combines_groups() {
        let result = merge_issues([
            RequiredValidator.validate(" ", &ValidationContext::new()),
            EmailValidator.validate("bad", &ValidationContext::new()),
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn range_validator_works_on_numbers() {
        let issues = RangeValidator::new(1, 5).validate(&10, &ValidationContext::new());
        assert_eq!(issues.len(), 1);
    }

    #[test]
    fn validation_error_converts_to_nest_error() {
        let err = ValidationError::from_issues_strict(vec![ValidationIssue::field_error(
            "email",
            "validation.email",
            "Invalid email",
        )])
        .unwrap_err();
        let nest_error: NestError = err.into();
        assert_eq!(nest_error.kind(), nest_error::NestErrorKind::Validation);
        assert_eq!(nest_error.code(), Some(NEST_VALIDATION_FAILED));
    }
}
