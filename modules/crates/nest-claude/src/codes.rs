//! Stable nest-claude error codes.

/// Generic Claude operation failure.
pub const NEST_CLAUDE_FAILED: &str = "NEST_CLAUDE_FAILED";

/// Claude configuration error.
pub const NEST_CLAUDE_CONFIG: &str = "NEST_CLAUDE_CONFIG";

/// Claude API key missing.
pub const NEST_CLAUDE_API_KEY_MISSING: &str = "NEST_CLAUDE_API_KEY_MISSING";

/// Claude HTTP request failure.
pub const NEST_CLAUDE_REQUEST_FAILED: &str = "NEST_CLAUDE_REQUEST_FAILED";

/// Claude response parse failure.
pub const NEST_CLAUDE_PARSE_FAILED: &str = "NEST_CLAUDE_PARSE_FAILED";

/// Claude rate limit exceeded (HTTP 429).
pub const NEST_CLAUDE_RATE_LIMITED: &str = "NEST_CLAUDE_RATE_LIMITED";

/// Claude API returned an `invalid_request_error` (HTTP 400).
pub const NEST_CLAUDE_INVALID_REQUEST: &str = "NEST_CLAUDE_INVALID_REQUEST";

/// Claude API returned an `authentication_error` (HTTP 401).
pub const NEST_CLAUDE_AUTH_FAILED: &str = "NEST_CLAUDE_AUTH_FAILED";

/// Claude API returned an `overloaded_error` (HTTP 529) or 5xx.
pub const NEST_CLAUDE_SERVER_ERROR: &str = "NEST_CLAUDE_SERVER_ERROR";

/// Claude API error response of an otherwise unclassified type.
pub const NEST_CLAUDE_API_ERROR: &str = "NEST_CLAUDE_API_ERROR";
