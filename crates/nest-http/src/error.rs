//! HTTP errors.

use std::error::Error;
use std::fmt;

use crate::codes::{NEST_HTTP_DECODE_FAILED, NEST_HTTP_REQUEST_FAILED, NEST_HTTP_TIMEOUT};
use crate::status::HttpStatus;

/// Result type for HTTP operations.
pub type HttpResult<T> = Result<T, HttpError>;

/// High-level category for an HTTP error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HttpErrorKind {
    /// Request timed out.
    Timeout,
    /// Connection or transport error.
    Connection,
    /// Non-success HTTP status.
    Status,
    /// Response body decode error.
    Decode,
    /// Authentication error.
    Auth,
    /// Configuration error.
    Config,
}

/// Structured error for nest-http and HTTP adapters.
#[derive(Debug)]
pub struct HttpError {
    kind: HttpErrorKind,
    message: String,
    code: Option<String>,
    status: Option<HttpStatus>,
    url: Option<String>,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl HttpError {
    /// Creates a new HTTP error.
    pub fn new(kind: HttpErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            code: None,
            status: None,
            url: None,
            source: None,
        }
    }

    /// Creates a timeout error.
    pub fn timeout(message: impl Into<String>) -> Self {
        Self::new(HttpErrorKind::Timeout, message).with_code(NEST_HTTP_TIMEOUT)
    }

    /// Creates a connection error.
    pub fn connection(message: impl Into<String>) -> Self {
        Self::new(HttpErrorKind::Connection, message)
    }

    /// Creates a non-success status error.
    pub fn from_status(status: HttpStatus, message: impl Into<String>) -> Self {
        Self::new(HttpErrorKind::Status, message)
            .with_code(NEST_HTTP_REQUEST_FAILED)
            .with_status(status)
    }

    /// Creates a decode error.
    pub fn decode(message: impl Into<String>) -> Self {
        Self::new(HttpErrorKind::Decode, message).with_code(NEST_HTTP_DECODE_FAILED)
    }

    /// Creates an auth error.
    pub fn auth(message: impl Into<String>) -> Self {
        Self::new(HttpErrorKind::Auth, message)
    }

    /// Creates a config error.
    pub fn config(message: impl Into<String>) -> Self {
        Self::new(HttpErrorKind::Config, message)
    }

    /// Sets a stable error code.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Sets the HTTP status context.
    pub fn with_status(mut self, status: HttpStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Sets the request URL context.
    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    /// Attaches a source error.
    pub fn with_source(mut self, source: impl Error + Send + Sync + 'static) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    /// Returns the error kind.
    pub fn kind(&self) -> HttpErrorKind {
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

    /// Returns the HTTP status context, if set.
    pub fn response_status(&self) -> Option<HttpStatus> {
        self.status
    }

    /// Returns the URL context, if set.
    pub fn url(&self) -> Option<&str> {
        self.url.as_deref()
    }

    /// Default code when converting to [`nest_error::NestError`].
    pub fn nest_code(&self) -> &str {
        self.code.as_deref().unwrap_or(NEST_HTTP_REQUEST_FAILED)
    }
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for HttpError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| source.as_ref() as &(dyn Error + 'static))
    }
}
