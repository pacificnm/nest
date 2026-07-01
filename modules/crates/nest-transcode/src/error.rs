//! Transcode and probe errors.

use std::error::Error;
use std::fmt;

use nest_error::NestError;
use nest_media::MediaError;

use crate::codes::{
    NEST_TRANSCODE_BINARY_NOT_FOUND, NEST_TRANSCODE_CONFIG, NEST_TRANSCODE_FAILED,
    NEST_TRANSCODE_IO_FAILED, NEST_TRANSCODE_PARSE_FAILED, NEST_TRANSCODE_PROBE_FAILED,
    NEST_TRANSCODE_TIMEOUT,
};

/// Result type for transcode / probe operations.
pub type TranscodeResult<T> = Result<T, TranscodeError>;

/// High-level category for a transcode error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TranscodeErrorKind {
    /// Configuration error.
    Config,
    /// FFprobe binary not found.
    BinaryNotFound,
    /// FFprobe probe failure.
    Probe,
    /// JSON parse failure.
    Parse,
    /// Probe timeout.
    Timeout,
    /// Filesystem I/O failure.
    Io,
}

/// Structured error for nest-transcode.
#[derive(Debug)]
pub struct TranscodeError {
    kind: TranscodeErrorKind,
    message: String,
    code: Option<String>,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl TranscodeError {
    /// Creates a new transcode error.
    pub fn new(kind: TranscodeErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            code: None,
            source: None,
        }
    }

    /// Creates a config error.
    pub fn config(message: impl Into<String>) -> Self {
        Self::new(TranscodeErrorKind::Config, message).with_code(NEST_TRANSCODE_CONFIG)
    }

    /// Creates a binary-not-found error.
    pub fn binary_not_found(message: impl Into<String>) -> Self {
        Self::new(TranscodeErrorKind::BinaryNotFound, message)
            .with_code(NEST_TRANSCODE_BINARY_NOT_FOUND)
    }

    /// Creates a probe error.
    pub fn probe(message: impl Into<String>) -> Self {
        Self::new(TranscodeErrorKind::Probe, message).with_code(NEST_TRANSCODE_PROBE_FAILED)
    }

    /// Creates a parse error.
    pub fn parse(message: impl Into<String>) -> Self {
        Self::new(TranscodeErrorKind::Parse, message).with_code(NEST_TRANSCODE_PARSE_FAILED)
    }

    /// Creates a timeout error.
    pub fn timeout(message: impl Into<String>) -> Self {
        Self::new(TranscodeErrorKind::Timeout, message).with_code(NEST_TRANSCODE_TIMEOUT)
    }

    /// Creates an I/O error.
    pub fn io(message: impl Into<String>) -> Self {
        Self::new(TranscodeErrorKind::Io, message).with_code(NEST_TRANSCODE_IO_FAILED)
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
    pub fn kind(&self) -> TranscodeErrorKind {
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
        self.code.as_deref().unwrap_or(NEST_TRANSCODE_FAILED)
    }
}

impl fmt::Display for TranscodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for TranscodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| source.as_ref() as &(dyn Error + 'static))
    }
}

impl From<NestError> for TranscodeError {
    fn from(error: NestError) -> Self {
        TranscodeError::io(error.to_string()).with_source(error)
    }
}

impl From<nest_file::FileError> for TranscodeError {
    fn from(error: nest_file::FileError) -> Self {
        TranscodeError::io(error.message()).with_source(error)
    }
}

/// Converts [`TranscodeError`] into [`MediaError`] for provider trait boundaries.
pub fn transcode_to_media_error(error: TranscodeError) -> MediaError {
    MediaError::inspection(error.message()).with_source(error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_error_maps_to_media_inspection() {
        let error = TranscodeError::probe("ffprobe failed");
        let media_error = transcode_to_media_error(error);
        assert_eq!(media_error.kind(), nest_media::MediaErrorKind::Inspection);
    }
}
