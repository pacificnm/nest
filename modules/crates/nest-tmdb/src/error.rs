//! TMDB-specific error mapping.

use std::error::Error;
use std::fmt;

use nest_error::NestError;
use nest_http::HttpError;
use nest_media::MediaError;

use crate::codes::{
    NEST_TMDB_API_ERROR, NEST_TMDB_CONFIG, NEST_TMDB_FAILED, NEST_TMDB_NOT_FOUND,
    NEST_TMDB_PARSE_FAILED, NEST_TMDB_RATE_LIMITED, NEST_TMDB_REQUEST_FAILED,
};

/// Result type for TMDB operations.
pub type TmdbResult<T> = Result<T, TmdbError>;

/// High-level category for a TMDB error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TmdbErrorKind {
    /// Configuration error.
    Config,
    /// HTTP transport failure.
    Http,
    /// TMDB API error response.
    Api,
    /// JSON parse failure.
    Parse,
    /// Resource not found.
    NotFound,
    /// Rate limit exceeded.
    RateLimit,
}

/// Structured error for nest-tmdb.
#[derive(Debug)]
pub struct TmdbError {
    kind: TmdbErrorKind,
    message: String,
    code: Option<String>,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl TmdbError {
    /// Creates a new TMDB error.
    pub fn new(kind: TmdbErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            code: None,
            source: None,
        }
    }

    /// Creates a config error.
    pub fn config(message: impl Into<String>) -> Self {
        Self::new(TmdbErrorKind::Config, message).with_code(NEST_TMDB_CONFIG)
    }

    /// Creates an HTTP error.
    pub fn http(message: impl Into<String>) -> Self {
        Self::new(TmdbErrorKind::Http, message).with_code(NEST_TMDB_REQUEST_FAILED)
    }

    /// Creates an API error.
    pub fn api(message: impl Into<String>) -> Self {
        Self::new(TmdbErrorKind::Api, message).with_code(NEST_TMDB_API_ERROR)
    }

    /// Creates a parse error.
    pub fn parse(message: impl Into<String>) -> Self {
        Self::new(TmdbErrorKind::Parse, message).with_code(NEST_TMDB_PARSE_FAILED)
    }

    /// Creates a not-found error.
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(TmdbErrorKind::NotFound, message).with_code(NEST_TMDB_NOT_FOUND)
    }

    /// Creates a rate-limit error.
    pub fn rate_limited(message: impl Into<String>) -> Self {
        Self::new(TmdbErrorKind::RateLimit, message).with_code(NEST_TMDB_RATE_LIMITED)
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
    pub fn kind(&self) -> TmdbErrorKind {
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
        self.code.as_deref().unwrap_or(NEST_TMDB_FAILED)
    }

    /// Converts to a [`MediaError`] for provider trait boundaries.
    pub fn into_media_error(self) -> MediaError {
        let message = self.message.clone();
        let kind = self.kind;
        let error = match kind {
            TmdbErrorKind::NotFound => MediaError::not_found(message),
            TmdbErrorKind::Config => MediaError::config(message),
            TmdbErrorKind::Parse
            | TmdbErrorKind::Api
            | TmdbErrorKind::Http
            | TmdbErrorKind::RateLimit => MediaError::provider(message),
        };
        if let Some(source) = self.source {
            error.with_source(TmdbError {
                kind,
                message: self.message,
                code: self.code,
                source: Some(source),
            })
        } else {
            error
        }
    }
}

impl fmt::Display for TmdbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for TmdbError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| source.as_ref() as &(dyn Error + 'static))
    }
}

impl From<NestError> for TmdbError {
    fn from(error: NestError) -> Self {
        if let Some(source) = error.source().and_then(|s| s.downcast_ref::<HttpError>()) {
            if source
                .response_status()
                .is_some_and(|status| status.code() == 404)
            {
                return TmdbError::not_found(error.to_string()).with_source(error);
            }
            if source
                .response_status()
                .is_some_and(|status| status.code() == 429)
            {
                return TmdbError::rate_limited("TMDB rate limit exceeded").with_source(error);
            }
        }
        TmdbError::http(error.to_string()).with_source(error)
    }
}

/// Maps a decode failure into a TMDB error.
#[allow(dead_code)]
pub fn invalid_response(message: impl Into<String>, operation: &str) -> TmdbError {
    TmdbError::parse(message).with_source(std::io::Error::new(
        std::io::ErrorKind::InvalidData,
        operation.to_string(),
    ))
}

/// Converts [`TmdbError`] into [`MediaError`] based on kind.
pub fn tmdb_to_media_error(error: TmdbError) -> MediaError {
    match error.kind() {
        TmdbErrorKind::NotFound => MediaError::not_found(error.message()).with_source(error),
        TmdbErrorKind::Config => MediaError::config(error.message()).with_source(error),
        _ => MediaError::provider(error.message()).with_source(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_maps_to_media_not_found() {
        let tmdb_error = TmdbError::not_found("movie missing");
        let media_error = tmdb_to_media_error(tmdb_error);
        assert_eq!(media_error.kind(), nest_media::MediaErrorKind::NotFound);
    }
}
