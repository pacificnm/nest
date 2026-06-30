//! HTTP response model.

use crate::headers::HeaderMap;
use crate::status::HttpStatus;

/// Raw HTTP response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    /// Response status.
    pub status: HttpStatus,
    /// Response headers.
    pub headers: HeaderMap,
    /// Response body bytes.
    pub body: Vec<u8>,
}

impl HttpResponse {
    /// Creates a new response.
    pub fn new(status: HttpStatus, body: impl Into<Vec<u8>>) -> Self {
        Self {
            status,
            headers: HeaderMap::new(),
            body: body.into(),
        }
    }

    /// Returns the body as UTF-8 text if valid.
    pub fn body_text(&self) -> Option<&str> {
        std::str::from_utf8(&self.body).ok()
    }
}

/// Standard API response envelope.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ApiResponse<T> {
    /// Response payload.
    pub data: T,
    /// Optional metadata.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub meta: Option<serde_json::Value>,
}

impl<T> ApiResponse<T> {
    /// Creates a response with data only.
    pub fn new(data: T) -> Self {
        Self { data, meta: None }
    }

    /// Creates a response with data and metadata.
    #[cfg(feature = "serde")]
    pub fn with_meta(data: T, meta: serde_json::Value) -> Self {
        Self {
            data,
            meta: Some(meta),
        }
    }
}
