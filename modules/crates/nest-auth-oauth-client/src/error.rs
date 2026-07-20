//! nest-auth-oauth-client errors.
//!
//! Distinct from [`nest_auth::AuthError`] on purpose: `AuthError` covers
//! `TokenStore` (storage) failures, while the OAuth2 protocol flow this
//! crate implements fails in ways storage never does (network/transport,
//! CSRF state mismatch, user denial). Converts to
//! [`nest_error::NestError`] via `impl From<OAuthError> for NestError` in
//! `lib.rs`, same as every other domain error in the framework.

use std::error::Error;
use std::fmt;

use crate::codes::{
    NEST_AUTH_OAUTH_ACCESS_DENIED, NEST_AUTH_OAUTH_CALLBACK_FAILED, NEST_AUTH_OAUTH_CONFIG,
    NEST_AUTH_OAUTH_FAILED, NEST_AUTH_OAUTH_PARSE_FAILED, NEST_AUTH_OAUTH_REQUEST_FAILED,
    NEST_AUTH_OAUTH_STATE_MISMATCH,
};

/// Result type for nest-auth-oauth-client operations.
pub type OAuthResult<T> = Result<T, OAuthError>;

/// High-level category for an OAuth client error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OAuthErrorKind {
    /// Invalid or missing configuration.
    Config,
    /// Transport/network failure talking to the authorization server.
    Request,
    /// Failed to parse a token response.
    Parse,
    /// Redirect callback `state` did not match the request's CSRF token.
    StateMismatch,
    /// The user denied the authorization request.
    AccessDenied,
    /// The loopback redirect callback failed.
    Callback,
}

/// Structured error for nest-auth-oauth-client.
#[derive(Debug)]
pub struct OAuthError {
    kind: OAuthErrorKind,
    message: String,
    code: Option<String>,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl OAuthError {
    /// Creates a new OAuth client error.
    pub fn new(kind: OAuthErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            code: None,
            source: None,
        }
    }

    /// Creates a config error.
    pub fn config(message: impl Into<String>) -> Self {
        Self::new(OAuthErrorKind::Config, message).with_code(NEST_AUTH_OAUTH_CONFIG)
    }

    /// Creates a request error.
    pub fn request(message: impl Into<String>) -> Self {
        Self::new(OAuthErrorKind::Request, message).with_code(NEST_AUTH_OAUTH_REQUEST_FAILED)
    }

    /// Creates a parse error.
    pub fn parse(message: impl Into<String>) -> Self {
        Self::new(OAuthErrorKind::Parse, message).with_code(NEST_AUTH_OAUTH_PARSE_FAILED)
    }

    /// Creates a state-mismatch (CSRF) error.
    pub fn state_mismatch(message: impl Into<String>) -> Self {
        Self::new(OAuthErrorKind::StateMismatch, message).with_code(NEST_AUTH_OAUTH_STATE_MISMATCH)
    }

    /// Creates an access-denied error.
    pub fn access_denied(message: impl Into<String>) -> Self {
        Self::new(OAuthErrorKind::AccessDenied, message).with_code(NEST_AUTH_OAUTH_ACCESS_DENIED)
    }

    /// Creates a callback-listener error.
    pub fn callback(message: impl Into<String>) -> Self {
        Self::new(OAuthErrorKind::Callback, message).with_code(NEST_AUTH_OAUTH_CALLBACK_FAILED)
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
    pub fn kind(&self) -> OAuthErrorKind {
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
        self.code.as_deref().unwrap_or(NEST_AUTH_OAUTH_FAILED)
    }
}

impl fmt::Display for OAuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for OAuthError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| source.as_ref() as &(dyn Error + 'static))
    }
}
