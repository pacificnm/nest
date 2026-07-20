//! Stable error codes for nest-auth-oauth-client.

/// Default code for an unclassified OAuth client error.
pub const NEST_AUTH_OAUTH_FAILED: &str = "NEST_AUTH_OAUTH_FAILED";
/// Invalid or missing OAuth client configuration.
pub const NEST_AUTH_OAUTH_CONFIG: &str = "NEST_AUTH_OAUTH_CONFIG";
/// Token request (exchange or refresh) failed against the authorization server.
pub const NEST_AUTH_OAUTH_REQUEST_FAILED: &str = "NEST_AUTH_OAUTH_REQUEST_FAILED";
/// Failed to parse a token response.
pub const NEST_AUTH_OAUTH_PARSE_FAILED: &str = "NEST_AUTH_OAUTH_PARSE_FAILED";
/// The redirect callback's `state` did not match the request's CSRF token.
pub const NEST_AUTH_OAUTH_STATE_MISMATCH: &str = "NEST_AUTH_OAUTH_STATE_MISMATCH";
/// The user denied the authorization request.
pub const NEST_AUTH_OAUTH_ACCESS_DENIED: &str = "NEST_AUTH_OAUTH_ACCESS_DENIED";
/// The loopback redirect callback failed (bind, accept, read, or malformed request).
pub const NEST_AUTH_OAUTH_CALLBACK_FAILED: &str = "NEST_AUTH_OAUTH_CALLBACK_FAILED";
