//! Validation errors and results.

use crate::issue::ValidationIssue;

/// Result of validating a value.
pub type ValidationResult = Result<(), ValidationError>;

/// Aggregated validation failure with one or more issues.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    issues: Vec<ValidationIssue>,
}

impl ValidationError {
    /// Creates a validation error from issues.
    ///
    /// Returns `Ok(())` when there are no issues. Returns `Err` when any issue
    /// has [`Severity::Error`]. Warnings and info without errors still return
    /// `Ok(())` via [`ValidationResult::Ok`] — use [`Self::from_issues_strict`]
    /// to treat any issue as failure.
    pub fn from_issues(issues: Vec<ValidationIssue>) -> ValidationResult {
        if issues.is_empty() {
            return Ok(());
        }
        if issues.iter().any(|issue| issue.is_blocking()) {
            Err(Self { issues })
        } else {
            Ok(())
        }
    }

    /// Creates an error from issues, failing if any issue exists regardless of severity.
    pub fn from_issues_strict(issues: Vec<ValidationIssue>) -> ValidationResult {
        if issues.is_empty() {
            Ok(())
        } else {
            Err(Self { issues })
        }
    }

    /// Returns all collected issues.
    pub fn issues(&self) -> &[ValidationIssue] {
        &self.issues
    }

    /// Returns only blocking error-severity issues.
    pub fn blocking_issues(&self) -> impl Iterator<Item = &ValidationIssue> {
        self.issues.iter().filter(|issue| issue.is_blocking())
    }

    /// Returns a short summary for conversion into [`nest_error::NestError`].
    pub fn summary_message(&self) -> String {
        let count = self.blocking_issues().count();
        if count == 1 {
            self.blocking_issues()
                .next()
                .map(|issue| issue.message.clone())
                .unwrap_or_else(|| "Validation failed".to_string())
        } else {
            format!("Validation failed with {count} errors")
        }
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.summary_message())
    }
}

impl std::error::Error for ValidationError {}
