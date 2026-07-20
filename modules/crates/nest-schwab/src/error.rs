//! nest-schwab errors.

use std::error::Error;
use std::fmt;

use nest_error::NestError;

use crate::codes::{
    NEST_SCHWAB_AUTH_FAILED, NEST_SCHWAB_CONFIG, NEST_SCHWAB_FAILED, NEST_SCHWAB_NOT_FOUND,
    NEST_SCHWAB_PARSE_FAILED, NEST_SCHWAB_REQUEST_FAILED,
};

/// Result type for Schwab client operations.
pub type SchwabResult<T> = Result<T, SchwabError>;

/// High-level category for a Schwab client error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SchwabErrorKind {
    /// Invalid or missing configuration.
    Config,
    /// Transport/network failure or non-success HTTP status.
    Request,
    /// Failed to parse a response body.
    Parse,
    /// Schwab rejected the request as unauthenticated/unauthorized.
    Auth,
    /// The requested resource was not found.
    NotFound,
}

/// Structured error for nest-schwab.
#[derive(Debug)]
pub struct SchwabError {
    kind: SchwabErrorKind,
    message: String,
    code: Option<String>,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl SchwabError {
    /// Creates a new Schwab client error.
    pub fn new(kind: SchwabErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            code: None,
            source: None,
        }
    }

    /// Creates a config error.
    pub fn config(message: impl Into<String>) -> Self {
        Self::new(SchwabErrorKind::Config, message).with_code(NEST_SCHWAB_CONFIG)
    }

    /// Creates a request error.
    pub fn request(message: impl Into<String>) -> Self {
        Self::new(SchwabErrorKind::Request, message).with_code(NEST_SCHWAB_REQUEST_FAILED)
    }

    /// Creates a parse error.
    pub fn parse(message: impl Into<String>) -> Self {
        Self::new(SchwabErrorKind::Parse, message).with_code(NEST_SCHWAB_PARSE_FAILED)
    }

    /// Creates an auth error.
    pub fn auth(message: impl Into<String>) -> Self {
        Self::new(SchwabErrorKind::Auth, message).with_code(NEST_SCHWAB_AUTH_FAILED)
    }

    /// Creates a not-found error.
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(SchwabErrorKind::NotFound, message).with_code(NEST_SCHWAB_NOT_FOUND)
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
    pub fn kind(&self) -> SchwabErrorKind {
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
        self.code.as_deref().unwrap_or(NEST_SCHWAB_FAILED)
    }
}

impl fmt::Display for SchwabError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for SchwabError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| source.as_ref() as &(dyn Error + 'static))
    }
}

impl From<NestError> for SchwabError {
    fn from(error: NestError) -> Self {
        SchwabError::request(error.to_string()).with_source(error)
    }
}
