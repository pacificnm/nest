//! UI/CLI-ready error report snapshot.

use crate::kind::NestErrorKind;
use crate::NestError;

/// A display-ready snapshot of a [`NestError`] for UI dialogs, CLI output, or logs.
///
/// Does not include the full source error chain; safe details are in [`details`](Self::details).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NestErrorReport {
    /// Human-readable title derived from error kind.
    pub title: String,
    /// Primary user-facing message.
    pub message: String,
    /// Error category.
    pub kind: NestErrorKind,
    /// Stable error code, if set.
    pub code: Option<String>,
    /// Recovery hint for the user.
    pub help: Option<String>,
    /// Additional context lines (module, operation, source summary).
    pub details: Vec<String>,
}

impl NestErrorReport {
    /// Creates a report from a [`NestError`].
    pub fn from_error(error: &NestError) -> Self {
        let mut details = Vec::new();

        if let Some(module) = error.module() {
            details.push(format!("module: {module}"));
        }
        if let Some(operation) = error.operation() {
            details.push(format!("operation: {operation}"));
        }
        if let Some(source) = error.source() {
            details.push(format!("source: {source}"));
        }

        Self {
            title: error.kind().title().to_string(),
            message: error.message().to_string(),
            kind: error.kind(),
            code: error.code().map(str::to_string),
            help: error.help().map(str::to_string),
            details,
        }
    }
}

impl From<&NestError> for NestErrorReport {
    fn from(error: &NestError) -> Self {
        Self::from_error(error)
    }
}

#[cfg(test)]
mod tests {
    use crate::NestError;

    #[test]
    fn report_includes_metadata() {
        let error = NestError::validation("Email is required")
            .with_code("NEST_VALIDATION_REQUIRED")
            .with_module("nest-forms")
            .with_operation("save_customer")
            .with_help("Enter a valid email.");

        let report = error.report();
        assert_eq!(report.title, "Validation Error");
        assert_eq!(report.message, "Email is required");
        assert_eq!(report.code.as_deref(), Some("NEST_VALIDATION_REQUIRED"));
        assert_eq!(report.help.as_deref(), Some("Enter a valid email."));
        assert!(report.details.iter().any(|d| d.contains("nest-forms")));
    }
}
