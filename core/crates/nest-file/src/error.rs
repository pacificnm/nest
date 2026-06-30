//! File errors.

use std::error::Error;
use std::fmt;
use std::path::PathBuf;

use crate::codes::{
    NEST_FILE_DELETE_FAILED, NEST_FILE_NOT_FOUND, NEST_FILE_PERMISSION_DENIED,
    NEST_FILE_READ_FAILED, NEST_FILE_WRITE_FAILED,
};

/// Result type for file operations.
pub type FileResult<T> = Result<T, FileError>;

/// High-level category for a file error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileErrorKind {
    /// File or directory not found.
    NotFound,
    /// Permission denied.
    PermissionDenied,
    /// Path validation failed.
    Path,
    /// Read failed.
    Read,
    /// Write failed.
    Write,
    /// Delete failed.
    Delete,
    /// Configuration error.
    Config,
}

/// Structured error for nest-file.
#[derive(Debug)]
pub struct FileError {
    kind: FileErrorKind,
    message: String,
    code: Option<String>,
    path: Option<PathBuf>,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl FileError {
    /// Creates a new file error.
    pub fn new(kind: FileErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            code: None,
            path: None,
            source: None,
        }
    }

    /// Creates a not-found error.
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(FileErrorKind::NotFound, message).with_code(NEST_FILE_NOT_FOUND)
    }

    /// Creates a permission denied error.
    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::new(FileErrorKind::PermissionDenied, message)
            .with_code(NEST_FILE_PERMISSION_DENIED)
    }

    /// Creates a path validation error.
    pub fn invalid_path(message: impl Into<String>) -> Self {
        Self::new(FileErrorKind::Path, message)
    }

    /// Creates a read error.
    pub fn read(message: impl Into<String>) -> Self {
        Self::new(FileErrorKind::Read, message).with_code(NEST_FILE_READ_FAILED)
    }

    /// Creates a write error.
    pub fn write(message: impl Into<String>) -> Self {
        Self::new(FileErrorKind::Write, message).with_code(NEST_FILE_WRITE_FAILED)
    }

    /// Creates a delete error.
    pub fn delete(message: impl Into<String>) -> Self {
        Self::new(FileErrorKind::Delete, message).with_code(NEST_FILE_DELETE_FAILED)
    }

    /// Creates a config error.
    pub fn config(message: impl Into<String>) -> Self {
        Self::new(FileErrorKind::Config, message)
    }

    /// Sets a stable error code.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Sets the path context.
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
    pub fn kind(&self) -> FileErrorKind {
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
        self.code.as_deref().unwrap_or(NEST_FILE_READ_FAILED)
    }
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for FileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| source.as_ref() as &(dyn Error + 'static))
    }
}

impl From<FileError> for nest_error::NestError {
    fn from(error: FileError) -> nest_error::NestError {
        let mut nest_error = nest_error::NestError::io(error.message())
            .with_code(error.nest_code())
            .with_module("nest-file");

        if let Some(path) = error.file_path() {
            nest_error = nest_error.with_operation(format!("path: {}", path.display()));
        }

        nest_error.with_source(error)
    }
}

pub(crate) fn map_io_error(error: std::io::Error, path: &std::path::Path) -> FileError {
    use std::io::ErrorKind;

    let base = match error.kind() {
        ErrorKind::NotFound => FileError::not_found(format!("not found: {}", path.display())),
        ErrorKind::PermissionDenied => {
            FileError::permission_denied(format!("permission denied: {}", path.display()))
        }
        _ => FileError::read(format!("I/O failed: {}", path.display())),
    };

    base.with_path(path).with_source(error)
}
