//! Media layer errors.

use std::error::Error;
use std::fmt;

use crate::codes::{
    NEST_MEDIA_CONFIG, NEST_MEDIA_FAILED, NEST_MEDIA_INSPECTION_FAILED, NEST_MEDIA_INVALID_INPUT,
    NEST_MEDIA_NOT_FOUND, NEST_MEDIA_PROVIDER_FAILED, NEST_MEDIA_REPOSITORY_FAILED,
};

/// Result type for media operations.
pub type MediaResult<T> = Result<T, MediaError>;

/// High-level category for a media error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MediaErrorKind {
    /// Entity not found.
    NotFound,
    /// Invalid input or field value.
    InvalidInput,
    /// Metadata provider failure.
    Provider,
    /// Library repository failure.
    Repository,
    /// Media inspection failure.
    Inspection,
    /// Configuration error.
    Config,
}

/// Structured error for nest-media and provider crates.
#[derive(Debug)]
pub struct MediaError {
    kind: MediaErrorKind,
    message: String,
    code: Option<String>,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl MediaError {
    /// Creates a new media error.
    pub fn new(kind: MediaErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            code: None,
            source: None,
        }
    }

    /// Creates a not-found error.
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(MediaErrorKind::NotFound, message).with_code(NEST_MEDIA_NOT_FOUND)
    }

    /// Creates an invalid-input error.
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::new(MediaErrorKind::InvalidInput, message).with_code(NEST_MEDIA_INVALID_INPUT)
    }

    /// Creates a provider error.
    pub fn provider(message: impl Into<String>) -> Self {
        Self::new(MediaErrorKind::Provider, message).with_code(NEST_MEDIA_PROVIDER_FAILED)
    }

    /// Creates a repository error.
    pub fn repository(message: impl Into<String>) -> Self {
        Self::new(MediaErrorKind::Repository, message).with_code(NEST_MEDIA_REPOSITORY_FAILED)
    }

    /// Creates an inspection error.
    pub fn inspection(message: impl Into<String>) -> Self {
        Self::new(MediaErrorKind::Inspection, message).with_code(NEST_MEDIA_INSPECTION_FAILED)
    }

    /// Creates a config error.
    pub fn config(message: impl Into<String>) -> Self {
        Self::new(MediaErrorKind::Config, message).with_code(NEST_MEDIA_CONFIG)
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
    pub fn kind(&self) -> MediaErrorKind {
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

    /// Default code when converting to [`nest_error::NestError`].
    pub fn nest_code(&self) -> &str {
        self.code.as_deref().unwrap_or(NEST_MEDIA_FAILED)
    }
}

impl fmt::Display for MediaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for MediaError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| source.as_ref() as &(dyn Error + 'static))
    }
}
