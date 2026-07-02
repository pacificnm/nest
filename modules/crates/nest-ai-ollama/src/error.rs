//! Ollama-specific error mapping.

use std::error::Error;
use std::fmt;

use nest_ai::AiError;
use nest_error::NestError;

use crate::codes::{
    NEST_AI_OLLAMA_CONFIG, NEST_AI_OLLAMA_FAILED, NEST_AI_OLLAMA_PARSE_FAILED,
    NEST_AI_OLLAMA_REQUEST_FAILED,
};

/// Result type for Ollama operations.
pub type OllamaResult<T> = Result<T, OllamaError>;

/// High-level category for an Ollama error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OllamaErrorKind {
    /// Configuration error.
    Config,
    /// HTTP transport failure.
    Http,
    /// JSON parse failure.
    Parse,
}

/// Structured error for nest-ai-ollama.
#[derive(Debug)]
pub struct OllamaError {
    kind: OllamaErrorKind,
    message: String,
    code: Option<String>,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl OllamaError {
    /// Creates a new Ollama error.
    pub fn new(kind: OllamaErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            code: None,
            source: None,
        }
    }

    /// Creates a config error.
    pub fn config(message: impl Into<String>) -> Self {
        Self::new(OllamaErrorKind::Config, message).with_code(NEST_AI_OLLAMA_CONFIG)
    }

    /// Creates an HTTP error.
    pub fn http(message: impl Into<String>) -> Self {
        Self::new(OllamaErrorKind::Http, message).with_code(NEST_AI_OLLAMA_REQUEST_FAILED)
    }

    /// Creates a parse error.
    pub fn parse(message: impl Into<String>) -> Self {
        Self::new(OllamaErrorKind::Parse, message).with_code(NEST_AI_OLLAMA_PARSE_FAILED)
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

    /// Returns the message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Converts to [`AiError`] for provider trait boundaries.
    pub fn into_ai_error(self) -> AiError {
        let message = self.message.clone();
        let error = match self.kind {
            OllamaErrorKind::Config => AiError::config(message),
            OllamaErrorKind::Http => AiError::request(message),
            OllamaErrorKind::Parse => AiError::parse(message),
        };
        if let Some(source) = self.source {
            error.with_source(OllamaError {
                kind: self.kind,
                message: self.message,
                code: self.code,
                source: Some(source),
            })
        } else {
            error
        }
    }
}

impl fmt::Display for OllamaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for OllamaError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| source.as_ref() as &(dyn Error + 'static))
    }
}

impl From<NestError> for OllamaError {
    fn from(error: NestError) -> Self {
        OllamaError::http(error.to_string()).with_source(error)
    }
}

/// Maps [`OllamaError`] into [`AiError`].
pub fn ollama_to_ai_error(error: OllamaError) -> AiError {
    error.into_ai_error()
}

/// Default code when converting to [`NestError`].
pub fn nest_code(error: &OllamaError) -> &str {
    error.code.as_deref().unwrap_or(NEST_AI_OLLAMA_FAILED)
}
