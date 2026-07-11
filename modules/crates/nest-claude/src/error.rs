//! Claude-specific error mapping.

use std::error::Error;
use std::fmt;

use nest_error::NestError;
use nest_http::HttpError;

use crate::codes::{
    NEST_CLAUDE_API_ERROR, NEST_CLAUDE_AUTH_FAILED, NEST_CLAUDE_CONFIG, NEST_CLAUDE_FAILED,
    NEST_CLAUDE_INVALID_REQUEST, NEST_CLAUDE_PARSE_FAILED, NEST_CLAUDE_RATE_LIMITED,
    NEST_CLAUDE_REQUEST_FAILED, NEST_CLAUDE_SERVER_ERROR,
};

/// Result type for Claude operations.
pub type ClaudeResult<T> = Result<T, ClaudeError>;

/// High-level category for a Claude error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClaudeErrorKind {
    /// Configuration error.
    Config,
    /// HTTP transport failure.
    Http,
    /// The API rejected the request shape (`invalid_request_error`, HTTP 400).
    InvalidRequest,
    /// Authentication failed (HTTP 401/403).
    Auth,
    /// Rate limit exceeded (HTTP 429).
    RateLimit,
    /// Anthropic-side server error (HTTP 5xx/529).
    Server,
    /// Response parse failure.
    Parse,
    /// Any other API error response.
    Api,
}

/// Structured error for nest-claude.
#[derive(Debug)]
pub struct ClaudeError {
    kind: ClaudeErrorKind,
    message: String,
    code: Option<String>,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl ClaudeError {
    /// Creates a new Claude error.
    pub fn new(kind: ClaudeErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            code: None,
            source: None,
        }
    }

    /// Creates a config error.
    pub fn config(message: impl Into<String>) -> Self {
        Self::new(ClaudeErrorKind::Config, message).with_code(NEST_CLAUDE_CONFIG)
    }

    /// Creates an HTTP transport error.
    pub fn http(message: impl Into<String>) -> Self {
        Self::new(ClaudeErrorKind::Http, message).with_code(NEST_CLAUDE_REQUEST_FAILED)
    }

    /// Creates a parse error.
    pub fn parse(message: impl Into<String>) -> Self {
        Self::new(ClaudeErrorKind::Parse, message).with_code(NEST_CLAUDE_PARSE_FAILED)
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
    pub fn kind(&self) -> ClaudeErrorKind {
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
        self.code.as_deref().unwrap_or(NEST_CLAUDE_FAILED)
    }
}

impl fmt::Display for ClaudeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for ClaudeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| source.as_ref() as &(dyn Error + 'static))
    }
}

impl From<NestError> for ClaudeError {
    fn from(error: NestError) -> Self {
        if let Some(source) = error.source().and_then(|s| s.downcast_ref::<HttpError>()) {
            if let Some(status) = source.response_status() {
                let code = status.code();
                let kind = match code {
                    400 => ClaudeErrorKind::InvalidRequest,
                    401 | 403 => ClaudeErrorKind::Auth,
                    429 => ClaudeErrorKind::RateLimit,
                    500..=599 => ClaudeErrorKind::Server,
                    _ => ClaudeErrorKind::Api,
                };
                let claude_code = match kind {
                    ClaudeErrorKind::InvalidRequest => NEST_CLAUDE_INVALID_REQUEST,
                    ClaudeErrorKind::Auth => NEST_CLAUDE_AUTH_FAILED,
                    ClaudeErrorKind::RateLimit => NEST_CLAUDE_RATE_LIMITED,
                    ClaudeErrorKind::Server => NEST_CLAUDE_SERVER_ERROR,
                    _ => NEST_CLAUDE_API_ERROR,
                };
                return ClaudeError::new(kind, error.to_string())
                    .with_code(claude_code)
                    .with_source(error);
            }
        }
        ClaudeError::http(error.to_string()).with_source(error)
    }
}

impl From<ClaudeError> for NestError {
    fn from(error: ClaudeError) -> Self {
        let kind = match error.kind() {
            ClaudeErrorKind::Config => nest_error::NestErrorKind::Config,
            ClaudeErrorKind::RateLimit | ClaudeErrorKind::Server | ClaudeErrorKind::Http => {
                nest_error::NestErrorKind::Network
            }
            ClaudeErrorKind::InvalidRequest
            | ClaudeErrorKind::Auth
            | ClaudeErrorKind::Parse
            | ClaudeErrorKind::Api => nest_error::NestErrorKind::Validation,
        };
        let code = error.nest_code().to_string();
        NestError::new(kind, error.to_string())
            .with_code(code)
            .with_module("nest-claude")
            .with_source(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_error_carries_message() {
        let error = ClaudeError::parse("bad json");
        assert_eq!(error.kind(), ClaudeErrorKind::Parse);
        assert_eq!(error.code(), Some(NEST_CLAUDE_PARSE_FAILED));
    }
}
