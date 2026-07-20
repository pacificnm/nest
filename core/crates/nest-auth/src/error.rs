//! nest-auth errors.

use std::error::Error;
use std::fmt;

use crate::codes::{NEST_AUTH_FAILED, NEST_AUTH_IO, NEST_AUTH_NOT_FOUND, NEST_AUTH_SERIALIZE};

/// Result type for auth operations.
pub type AuthResult<T> = Result<T, AuthError>;

/// High-level category for an auth error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuthErrorKind {
    /// No token stored under the requested key.
    NotFound,
    /// Underlying storage I/O failure (file, keyring, etc.).
    Io,
    /// Token serialization/deserialization failure.
    Serialize,
}

/// Structured error for nest-auth and its provider crates (mirrors
/// `nest_ai::AiError`'s shape — see that crate if this one's conventions
/// need cross-checking).
#[derive(Debug)]
pub struct AuthError {
    kind: AuthErrorKind,
    message: String,
    code: Option<String>,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl AuthError {
    /// Creates a new auth error.
    pub fn new(kind: AuthErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            code: None,
            source: None,
        }
    }

    /// Creates a not-found error.
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(AuthErrorKind::NotFound, message).with_code(NEST_AUTH_NOT_FOUND)
    }

    /// Creates an I/O error.
    pub fn io(message: impl Into<String>) -> Self {
        Self::new(AuthErrorKind::Io, message).with_code(NEST_AUTH_IO)
    }

    /// Creates a serialization error.
    pub fn serialize(message: impl Into<String>) -> Self {
        Self::new(AuthErrorKind::Serialize, message).with_code(NEST_AUTH_SERIALIZE)
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
    pub fn kind(&self) -> AuthErrorKind {
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

    /// Default code when a caller needs one and none was explicitly set.
    pub fn nest_code(&self) -> &str {
        self.code.as_deref().unwrap_or(NEST_AUTH_FAILED)
    }
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for AuthError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| source.as_ref() as &(dyn Error + 'static))
    }
}
