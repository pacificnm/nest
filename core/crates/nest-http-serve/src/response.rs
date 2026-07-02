//! Response helpers for handlers.

use std::convert::Infallible;

use axum::body::Body;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use bytes::Bytes;
use futures_core::Stream;
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
#[derive(Debug)]
pub struct HttpResponse {
    inner: HttpResponseInner,
}

#[derive(Debug)]
enum HttpResponseInner {
    Buffered {
        status: HttpStatus,
        headers: HeaderMap,
        body: Bytes,
    },
    Raw(Response),
}

impl HttpResponse {
    /// Creates a response with the given status and body.
    pub fn new(status: HttpStatus, body: impl Into<Bytes>) -> Self {
        Self {
            inner: HttpResponseInner::Buffered {
                status,
                headers: HeaderMap::new(),
                body: body.into(),
            },
        }
    }

    /// Creates an empty response.
    pub fn empty(status: HttpStatus) -> Self {
        Self::new(status, Bytes::new())
    }

    /// Wraps a pre-built axum response (for streaming/SSE handlers).
    pub fn raw(response: Response) -> Self {
        Self {
            inner: HttpResponseInner::Raw(response),
        }
    }

    /// Sets a response header.
    pub fn with_header(mut self, name: impl AsRef<str>, value: impl AsRef<str>) -> Self {
        if let HttpResponseInner::Buffered { headers, .. } = &mut self.inner {
            headers.insert(name, value.as_ref());
        }
        self
    }

    /// Returns the status code.
    pub fn status(&self) -> HttpStatus {
        match &self.inner {
            HttpResponseInner::Buffered { status, .. } => *status,
            HttpResponseInner::Raw(response) => HttpStatus(response.status().as_u16()),
        }
    }

    /// Creates a Server-Sent Events response from a byte stream.
    pub fn event_stream<S>(stream: S) -> HttpResult
    where
        S: Stream<Item = Result<Bytes, Infallible>> + Send + 'static,
    {
        let mut response = Response::new(Body::from_stream(stream));
        *response.status_mut() = StatusCode::OK;
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/event-stream"),
        );
        response
            .headers_mut()
            .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
        response
            .headers_mut()
            .insert(header::CONNECTION, HeaderValue::from_static("keep-alive"));
        Ok(Self::raw(response))
    }
}

/// Result type for route handlers and middleware.
pub type HttpResult = Result<HttpResponse, ServeError>;

impl<T: Serialize> Json<T> {
    /// Converts this JSON value into an HTTP response.
    pub fn into_response(self) -> HttpResult {
        self.into_response_with_status(HttpStatus::OK)
    }

    /// Converts this JSON value into an HTTP response with a custom status.
    pub fn into_response_with_status(self, status: HttpStatus) -> HttpResult {
        let body = serde_json::to_vec(&self.0)
            .map_err(|error| ServeError::from(nest_http::HttpError::decode(error.to_string())))?;
        Ok(HttpResponse::new(status, Bytes::from(body))
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
        match self.inner {
            HttpResponseInner::Raw(response) => response,
            HttpResponseInner::Buffered {
                status,
                headers,
                body,
            } => {
                let mut response = Response::new(Body::from(body));
                *response.status_mut() = StatusCode::from_u16(status.code())
                    .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

                let response_headers = response.headers_mut();
                for (name, value) in headers.iter() {
                    if let (Ok(name), Ok(value)) = (
                        header::HeaderName::from_bytes(name.as_bytes()),
                        HeaderValue::from_str(value),
                    ) {
                        response_headers.insert(name, value);
                    }
                }
                response
            }
        }
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
            br#"{"error":{"code":"NEST_HTTP_REQUEST_FAILED","message":"internal error"}}"#.to_vec()
        });
        let mut response = Response::new(Body::from(json));
        *response.status_mut() =
            StatusCode::from_u16(status.code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
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
