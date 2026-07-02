//! Handler error mapping to HTTP responses.

use nest_error::{NestError, NestErrorKind};
use nest_http::{HttpError, HttpStatus};

use crate::codes::{
    NEST_HTTP_SERVE_JSON_INVALID, NEST_HTTP_SERVE_PARAM_MISSING, NEST_HTTP_SERVE_QUERY_MISSING,
};
use crate::response::ErrorBody;

/// Errors returned from handlers and middleware.
#[derive(Debug)]
pub enum ServeError {
    /// HTTP-layer error from nest-http contracts.
    Http(HttpError),
    /// Nest framework error.
    Nest(NestError),
}

impl ServeError {
    /// Creates a bad-request error for a missing path parameter.
    pub fn param_missing(name: impl Into<String>) -> Self {
        let name = name.into();
        Self::Http(
            HttpError::from_status(
                HttpStatus::BAD_REQUEST,
                format!("missing path parameter: {name}"),
            )
            .with_code(NEST_HTTP_SERVE_PARAM_MISSING),
        )
    }

    /// Creates a bad-request error for a missing query parameter.
    pub fn query_missing(name: impl Into<String>) -> Self {
        let name = name.into();
        Self::Http(
            HttpError::from_status(
                HttpStatus::BAD_REQUEST,
                format!("missing query parameter: {name}"),
            )
            .with_code(NEST_HTTP_SERVE_QUERY_MISSING),
        )
    }

    /// Creates a bad-request error for invalid JSON.
    pub fn json_invalid(message: impl Into<String>) -> Self {
        Self::Http(HttpError::decode(message).with_code(NEST_HTTP_SERVE_JSON_INVALID))
    }

    /// Returns the HTTP status for this error.
    pub fn status(&self) -> HttpStatus {
        match self {
            Self::Http(error) => error
                .response_status()
                .unwrap_or(HttpStatus::INTERNAL_SERVER_ERROR),
            Self::Nest(error) => nest_error_status(error),
        }
    }

    /// Returns a stable error code.
    pub fn code(&self) -> &str {
        match self {
            Self::Http(error) => error.nest_code(),
            Self::Nest(error) => error.code().unwrap_or("NEST_UNKNOWN"),
        }
    }

    /// Returns the error message.
    pub fn message(&self) -> String {
        match self {
            Self::Http(error) => error.message().to_string(),
            Self::Nest(error) => error.to_string(),
        }
    }

    /// Converts to a JSON error body.
    pub fn into_body(self) -> ErrorBody {
        ErrorBody {
            code: self.code().to_string(),
            message: self.message(),
        }
    }
}

impl From<HttpError> for ServeError {
    fn from(error: HttpError) -> Self {
        Self::Http(error)
    }
}

impl From<NestError> for ServeError {
    fn from(error: NestError) -> Self {
        Self::Nest(error)
    }
}

fn nest_error_status(error: &NestError) -> HttpStatus {
    match error.kind() {
        NestErrorKind::Validation | NestErrorKind::Command => HttpStatus::BAD_REQUEST,
        NestErrorKind::Auth => HttpStatus::UNAUTHORIZED,
        NestErrorKind::Data => HttpStatus::NOT_FOUND,
        NestErrorKind::Network => HttpStatus::SERVICE_UNAVAILABLE,
        _ => HttpStatus::INTERNAL_SERVER_ERROR,
    }
}

/// Converts an [`HttpError`] into a [`NestError`].
pub fn http_error_to_nest_error(error: HttpError) -> NestError {
    let mut nest_error = NestError::network(error.message())
        .with_code(error.nest_code())
        .with_module("nest-http-serve");

    if let Some(status) = error.response_status() {
        nest_error = nest_error.with_help(format!("HTTP status: {}", status.code()));
    }

    nest_error.with_source(error)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nest_validation_maps_to_400() {
        let error = ServeError::from(NestError::validation("bad input"));
        assert_eq!(error.status(), HttpStatus::BAD_REQUEST);
    }
}
