//! HTTP request model.

use crate::correlation::{CorrelationId, RequestId};
use crate::headers::HeaderMap;
use crate::method::HttpMethod;

/// Lightweight HTTP request description.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpRequest {
    /// HTTP method.
    pub method: HttpMethod,
    /// Request URL.
    pub url: String,
    /// Request headers.
    pub headers: HeaderMap,
    /// Optional request body (raw bytes).
    pub body: Option<Vec<u8>>,
    /// Optional request id.
    pub request_id: Option<RequestId>,
    /// Optional correlation id.
    pub correlation_id: Option<CorrelationId>,
}

impl HttpRequest {
    /// Creates a new GET request.
    pub fn get(url: impl Into<String>) -> Self {
        Self::new(HttpMethod::Get, url)
    }

    /// Creates a new POST request.
    pub fn post(url: impl Into<String>) -> Self {
        Self::new(HttpMethod::Post, url)
    }

    /// Creates a new request with method and URL.
    pub fn new(method: HttpMethod, url: impl Into<String>) -> Self {
        Self {
            method,
            url: url.into(),
            headers: HeaderMap::new(),
            body: None,
            request_id: None,
            correlation_id: None,
        }
    }

    /// Sets the request body.
    pub fn with_body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = Some(body.into());
        self
    }

    /// Sets a header.
    pub fn with_header(mut self, name: impl AsRef<str>, value: impl Into<String>) -> Self {
        self.headers.insert(name, value);
        self
    }

    /// Sets the request id.
    pub fn with_request_id(mut self, id: RequestId) -> Self {
        self.request_id = Some(id);
        self
    }

    /// Sets the correlation id.
    pub fn with_correlation_id(mut self, id: CorrelationId) -> Self {
        self.correlation_id = Some(id);
        self
    }
}
