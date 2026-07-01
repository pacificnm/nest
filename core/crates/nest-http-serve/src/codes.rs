//! Stable serve-layer error codes.

/// Missing required path parameter.
pub const NEST_HTTP_SERVE_PARAM_MISSING: &str = "NEST_HTTP_SERVE_PARAM_MISSING";

/// Missing required query parameter.
pub const NEST_HTTP_SERVE_QUERY_MISSING: &str = "NEST_HTTP_SERVE_QUERY_MISSING";

/// Invalid JSON request body.
pub const NEST_HTTP_SERVE_JSON_INVALID: &str = "NEST_HTTP_SERVE_JSON_INVALID";

/// Server configuration error.
pub const NEST_HTTP_SERVE_CONFIG: &str = "NEST_HTTP_SERVE_CONFIG";
