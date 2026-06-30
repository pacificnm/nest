//! Validation issues, severity, and field paths.

use std::fmt;

/// Severity of a validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Severity {
    /// Blocks submission or persistence.
    Error,
    /// Soft validation; host may allow proceed.
    Warning,
    /// Informational suggestion.
    Info,
}

/// Dot/bracket path to a validated field.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FieldPath(String);

impl FieldPath {
    /// Creates a field path from a string.
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    /// Returns the path string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for FieldPath {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for FieldPath {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl fmt::Display for FieldPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A single structured validation finding.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ValidationIssue {
    /// Field path, if the issue applies to one field.
    pub field: Option<FieldPath>,
    /// Stable validation code (e.g. `validation.email`).
    pub code: String,
    /// Human-readable message.
    pub message: String,
    /// Issue severity.
    pub severity: Severity,
    /// Optional recovery hint.
    pub help: Option<String>,
}

impl ValidationIssue {
    /// Creates an error-level issue without a field path.
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: None,
            code: code.into(),
            message: message.into(),
            severity: Severity::Error,
            help: None,
        }
    }

    /// Creates an error-level issue for a field.
    pub fn field_error(
        field: impl Into<FieldPath>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            field: Some(field.into()),
            code: code.into(),
            message: message.into(),
            severity: Severity::Error,
            help: None,
        }
    }

    /// Creates a warning-level issue for a field.
    pub fn field_warning(
        field: impl Into<FieldPath>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            field: Some(field.into()),
            code: code.into(),
            message: message.into(),
            severity: Severity::Warning,
            help: None,
        }
    }

    /// Creates an info-level issue for a field.
    pub fn field_info(
        field: impl Into<FieldPath>,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            field: Some(field.into()),
            code: code.into(),
            message: message.into(),
            severity: Severity::Info,
            help: None,
        }
    }

    /// Attaches a recovery hint.
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Returns whether this issue blocks validation.
    pub fn is_blocking(&self) -> bool {
        self.severity == Severity::Error
    }
}
