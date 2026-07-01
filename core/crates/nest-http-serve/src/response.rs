//! Response helpers for handlers.

use axum::body::Body;
use axum::http::{header, HeaderMap as AxumHeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use bytes::Bytes;
use serde::Serialize;

use nest_http::{HeaderMap, HttpStatus};

use crate::error::ServeError;

/// JSON error envelope field.
#[derive(Debug, Clone, Serialize)]
pub struct ErrorBody {
    /// Stable error code.
    pub code: String,
    /// Human-readable message.
    pub message: String,
}

/// JSON error response wrapper.
#[derive(Debug, Clone, Serialize)]
struct ErrorEnvelope {
    error: ErrorBody,
}

/// JSON response wrapper.
#[derive(Debug, Clone)]
pub struct Json<T>(pub T);

impl<T: Serialize> Json<T> {
    /// Creates a JSON response with the default 200 status.
    pub fn new(value: T) -> Self {
        Self(value)
    }
}

/// Serve-layer HTTP response.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    status: HttpStatus,
    headers: HeaderMap,
    body: Bytes,
}

impl HttpResponse {
    /// Creates a response with the given status and body.
    pub fn new(status: HttpStatus, body: impl Into<Bytes>) -> Self {
        Self {
            status,
            headers: HeaderMap::new(),
            body: body.into(),
        }
    }

    /// Creates an empty response.
    pub fn empty(status: HttpStatus) -> Self {
        Self::new(status, Bytes::new())
    }

    /// Sets a response header.
    pub fn with_header(mut self, name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        self.headers.insert(name, value.as_ref());
        self
    }

    /// Returns the status code.
    pub fn status(&self) -> HttpStatus {
        self.status
    }
}

/// Result type for route handlers and middleware.
pub type HttpResult = Result<HttpResponse, ServeError>;

impl<T: Serialize> Json<T> {
    /// Converts this JSON value into an HTTP response.
    pub fn into_response(self) -> HttpResult {
        let body = serde_json::to_vec(&self.0).map_err(|error| {
            ServeError::from(nest_http::HttpError::decode(error.to_string()))
        })?;
        Ok(HttpResponse::new(HttpStatus::OK, Bytes::from(body))
            .with_header(header::CONTENT_TYPE.as_str(), "application/json"))
    }
}

impl<T: Serialize> From<Json<T>> for HttpResult {
    fn from(value: Json<T>) -> Self {
        value.into_response()
    }
}

impl HttpResponse {
    /// Converts into an axum response.
    pub fn into_axum_response(self) -> Response {
        let mut response = Response::new(Body::from(self.body));
        *response.status_mut() = StatusCode::from_u16(self.status.code())
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

        let headers = response.headers_mut();
        for (name, value) in self.headers.iter() {
            if let (Ok(name), Ok(value)) = (
                header::HeaderName::from_bytes(name.as_bytes()),
                HeaderValue::from_str(value),
            ) {
                headers.insert(name, value);
            }
        }
        response
    }
}

impl ServeError {
    /// Converts into an axum JSON error response.
    pub fn into_axum_response(self) -> Response {
        let status = self.status();
        let body = ErrorEnvelope {
            error: self.into_body(),
        };
        let json = serde_json::to_vec(&body).unwrap_or_else(|_| {
            br#"{"error":{"code":"NEST_HTTP_REQUEST_FAILED","message":"internal error"}}"#
                .to_vec()
        });
        let mut response = Response::new(Body::from(json));
        *response.status_mut() = StatusCode::from_u16(status.code())
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        response
    }
}

impl IntoResponse for HttpResponse {
    fn into_response(self) -> Response {
        self.into_axum_response()
    }
}

impl IntoResponse for ServeError {
    fn into_response(self) -> Response {
        self.into_axum_response()
    }
}

/// Converts a handler result into an axum response.
pub fn into_axum_response(result: HttpResult) -> Response {
    match result {
        Ok(response) => response.into_response(),
        Err(error) => error.into_response(),
    }
}
