//! Media library errors.

use std::error::Error;
use std::fmt;

use nest_error::NestError;
use nest_file::FileError;

use crate::codes::{
    NEST_MEDIA_LIBRARY_CONFIG, NEST_MEDIA_LIBRARY_FAILED, NEST_MEDIA_LIBRARY_INSPECTION_FAILED,
    NEST_MEDIA_LIBRARY_IO_FAILED, NEST_MEDIA_LIBRARY_PROVIDER_FAILED,
    NEST_MEDIA_LIBRARY_REPOSITORY_FAILED, NEST_MEDIA_LIBRARY_SCAN_FAILED,
};

/// Result type for media library operations.
pub type LibraryResult<T> = Result<T, LibraryError>;

/// High-level category for a library error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LibraryErrorKind {
    /// Filesystem scan failure.
    Scan,
    /// Configuration error.
    Config,
    /// Metadata provider failure.
    Provider,
    /// Repository failure.
    Repository,
    /// Media inspection failure.
    Inspection,
    /// Filesystem I/O failure.
    Io,
}

/// Structured error for nest-media-library.
#[derive(Debug)]
pub struct LibraryError {
    kind: LibraryErrorKind,
    message: String,
    code: Option<String>,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl LibraryError {
    /// Creates a new library error.
    pub fn new(kind: LibraryErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            code: None,
            source: None,
        }
    }

    /// Creates a scan error.
    pub fn scan(message: impl Into<String>) -> Self {
        Self::new(LibraryErrorKind::Scan, message).with_code(NEST_MEDIA_LIBRARY_SCAN_FAILED)
    }

    /// Creates a config error.
    pub fn config(message: impl Into<String>) -> Self {
        Self::new(LibraryErrorKind::Config, message).with_code(NEST_MEDIA_LIBRARY_CONFIG)
    }

    /// Creates a provider error.
    pub fn provider(message: impl Into<String>) -> Self {
        Self::new(LibraryErrorKind::Provider, message)
            .with_code(NEST_MEDIA_LIBRARY_PROVIDER_FAILED)
    }

    /// Creates a repository error.
    pub fn repository(message: impl Into<String>) -> Self {
        Self::new(LibraryErrorKind::Repository, message)
            .with_code(NEST_MEDIA_LIBRARY_REPOSITORY_FAILED)
    }

    /// Creates an inspection error.
    pub fn inspection(message: impl Into<String>) -> Self {
        Self::new(LibraryErrorKind::Inspection, message)
            .with_code(NEST_MEDIA_LIBRARY_INSPECTION_FAILED)
    }

    /// Creates an I/O error.
    pub fn io(message: impl Into<String>) -> Self {
        Self::new(LibraryErrorKind::Io, message).with_code(NEST_MEDIA_LIBRARY_IO_FAILED)
    }

    /// Sets a stable error code.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Attaches a source error.
    pub fn with_source(mut self, source: impl Error + Send + Sync + 'static) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    /// Returns the error kind.
    pub fn kind(&self) -> LibraryErrorKind {
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

    /// Default code when converting to [`NestError`].
    pub fn nest_code(&self) -> &str {
        self.code.as_deref().unwrap_or(NEST_MEDIA_LIBRARY_FAILED)
    }
}

impl fmt::Display for LibraryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for LibraryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| source.as_ref() as &(dyn Error + 'static))
    }
}

impl From<FileError> for LibraryError {
    fn from(error: FileError) -> Self {
        LibraryError::io(error.message()).with_source(error)
    }
}

impl From<NestError> for LibraryError {
    fn from(error: NestError) -> Self {
        LibraryError::io(error.to_string()).with_source(error)
    }
}

impl From<nest_media::MediaError> for LibraryError {
    fn from(error: nest_media::MediaError) -> Self {
        match error.kind() {
            nest_media::MediaErrorKind::Provider => {
                LibraryError::provider(error.message()).with_source(error)
            }
            nest_media::MediaErrorKind::Inspection => {
                LibraryError::inspection(error.message()).with_source(error)
            }
            nest_media::MediaErrorKind::Repository => {
                LibraryError::repository(error.message()).with_source(error)
            }
            _ => LibraryError::scan(error.message()).with_source(error),
        }
    }
}
