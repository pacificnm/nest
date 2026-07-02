//! CSV errors.

use std::error::Error;
use std::fmt;
use std::path::PathBuf;

use crate::codes::{
    NEST_CSV_DESERIALIZE_FAILED, NEST_CSV_PARSE_FAILED, NEST_CSV_REQUIRED_COLUMN_MISSING,
    NEST_CSV_ROW_LIMIT_EXCEEDED, NEST_CSV_VALIDATION_FAILED, NEST_CSV_WRITE_FAILED,
};

/// Result type for CSV operations.
pub type CsvResult<T> = Result<T, CsvError>;

/// High-level category for a CSV error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CsvErrorKind {
    /// CSV parse error.
    Parse,
    /// Row deserialize error.
    Deserialize,
    /// Required column missing.
    RequiredColumn,
    /// Row validation error.
    Validation,
    /// Write error.
    Write,
    /// Error limit exceeded.
    RowLimit,
    /// Configuration error.
    Config,
}

/// Structured error for nest-file-csv.
#[derive(Debug)]
pub struct CsvError {
    kind: CsvErrorKind,
    message: String,
    code: Option<String>,
    path: Option<PathBuf>,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl CsvError {
    /// Creates a new CSV error.
    pub fn new(kind: CsvErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            code: None,
            path: None,
            source: None,
        }
    }

    /// Creates a parse error.
    pub fn parse(message: impl Into<String>) -> Self {
        Self::new(CsvErrorKind::Parse, message).with_code(NEST_CSV_PARSE_FAILED)
    }

    /// Creates a deserialize error.
    pub fn deserialize(message: impl Into<String>) -> Self {
        Self::new(CsvErrorKind::Deserialize, message).with_code(NEST_CSV_DESERIALIZE_FAILED)
    }

    /// Creates a required-column error.
    pub fn required_column(message: impl Into<String>) -> Self {
        Self::new(CsvErrorKind::RequiredColumn, message).with_code(NEST_CSV_REQUIRED_COLUMN_MISSING)
    }

    /// Creates a validation error.
    pub fn validation(message: impl Into<String>) -> Self {
        Self::new(CsvErrorKind::Validation, message).with_code(NEST_CSV_VALIDATION_FAILED)
    }

    /// Creates a write error.
    pub fn write(message: impl Into<String>) -> Self {
        Self::new(CsvErrorKind::Write, message).with_code(NEST_CSV_WRITE_FAILED)
    }

    /// Creates a row-limit error.
    pub fn row_limit(message: impl Into<String>) -> Self {
        Self::new(CsvErrorKind::RowLimit, message).with_code(NEST_CSV_ROW_LIMIT_EXCEEDED)
    }

    /// Creates a config error.
    pub fn config(message: impl Into<String>) -> Self {
        Self::new(CsvErrorKind::Config, message)
    }

    /// Sets a stable error code.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Sets the file path context.
    pub fn with_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Attaches a source error.
    pub fn with_source(mut self, source: impl Error + Send + Sync + 'static) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    /// Returns the error kind.
    pub fn kind(&self) -> CsvErrorKind {
        self.kind
    }

    /// Returns the message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the stable code, if set.
    pub fn code(&self) -> Option<&str> {
        self.code.as_deref()
    }

    /// Returns the path context, if set.
    pub fn file_path(&self) -> Option<&PathBuf> {
        self.path.as_ref()
    }

    /// Default code when converting to [`nest_error::NestError`].
    pub fn nest_code(&self) -> &str {
        self.code.as_deref().unwrap_or(NEST_CSV_PARSE_FAILED)
    }
}

impl fmt::Display for CsvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for CsvError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| source.as_ref() as &(dyn Error + 'static))
    }
}

impl From<CsvError> for nest_error::NestError {
    fn from(error: CsvError) -> nest_error::NestError {
        let nest_error = match error.kind() {
            CsvErrorKind::Write => nest_error::NestError::io(error.message()),
            _ => nest_error::NestError::validation(error.message()),
        };

        let mut nest_error = nest_error
            .with_code(error.nest_code())
            .with_module("nest-file-csv");

        if let Some(path) = error.file_path() {
            nest_error = nest_error.with_operation(format!("path: {}", path.display()));
        }

        nest_error.with_source(error)
    }
}
