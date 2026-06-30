//! Stable HTTP error codes.

/// Generic HTTP request failure.
pub const NEST_HTTP_REQUEST_FAILED: &str = "NEST_HTTP_REQUEST_FAILED";

/// HTTP request timed out.
pub const NEST_HTTP_TIMEOUT: &str = "NEST_HTTP_TIMEOUT";

/// HTTP response body decode failed.
pub const NEST_HTTP_DECODE_FAILED: &str = "NEST_HTTP_DECODE_FAILED";
