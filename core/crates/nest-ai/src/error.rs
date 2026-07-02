//! AI layer errors.

use std::error::Error;
use std::fmt;

use crate::codes::{
    NEST_AI_CONFIG, NEST_AI_FAILED, NEST_AI_INVALID_INPUT, NEST_AI_PARSE_FAILED,
    NEST_AI_REQUEST_FAILED,
};

/// Result type for AI operations.
pub type AiResult<T> = Result<T, AiError>;

/// High-level category for an AI error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AiErrorKind {
    /// Invalid input or field value.
    InvalidInput,
    /// Provider HTTP or transport failure.
    Request,
    /// Response parse failure.
    Parse,
    /// Configuration error.
    Config,
}

/// Structured error for nest-ai and provider crates.
#[derive(Debug)]
pub struct AiError {
    kind: AiErrorKind,
    message: String,
    code: Option<String>,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl AiError {
    /// Creates a new AI error.
    pub fn new(kind: AiErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            code: None,
            source: None,
        }
    }

    /// Creates an invalid-input error.
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::new(AiErrorKind::InvalidInput, message).with_code(NEST_AI_INVALID_INPUT)
    }

    /// Creates a request error.
    pub fn request(message: impl Into<String>) -> Self {
        Self::new(AiErrorKind::Request, message).with_code(NEST_AI_REQUEST_FAILED)
    }

    /// Creates a parse error.
    pub fn parse(message: impl Into<String>) -> Self {
        Self::new(AiErrorKind::Parse, message).with_code(NEST_AI_PARSE_FAILED)
    }

    /// Creates a config error.
    pub fn config(message: impl Into<String>) -> Self {
        Self::new(AiErrorKind::Config, message).with_code(NEST_AI_CONFIG)
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
    pub fn kind(&self) -> AiErrorKind {
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
        self.code.as_deref().unwrap_or(NEST_AI_FAILED)
    }
}

impl fmt::Display for AiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for AiError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| source.as_ref() as &(dyn Error + 'static))
    }
}
