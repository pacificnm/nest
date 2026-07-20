//! Stable error codes for nest-schwab.

/// Default code for an unclassified Schwab client error.
pub const NEST_SCHWAB_FAILED: &str = "NEST_SCHWAB_FAILED";
/// Invalid or missing Schwab client configuration.
pub const NEST_SCHWAB_CONFIG: &str = "NEST_SCHWAB_CONFIG";
/// A request to the Schwab API failed (transport, non-success status).
pub const NEST_SCHWAB_REQUEST_FAILED: &str = "NEST_SCHWAB_REQUEST_FAILED";
/// Failed to parse a Schwab API response body.
pub const NEST_SCHWAB_PARSE_FAILED: &str = "NEST_SCHWAB_PARSE_FAILED";
/// The Schwab API rejected the request as unauthenticated/unauthorized.
pub const NEST_SCHWAB_AUTH_FAILED: &str = "NEST_SCHWAB_AUTH_FAILED";
/// The requested resource (account, order, etc.) was not found.
pub const NEST_SCHWAB_NOT_FOUND: &str = "NEST_SCHWAB_NOT_FOUND";
