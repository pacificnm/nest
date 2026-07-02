//! Stable error codes for nest-ai.

/// Generic AI operation failure.
pub const NEST_AI_FAILED: &str = "NEST_AI_FAILED";
/// Provider HTTP or transport failure.
pub const NEST_AI_REQUEST_FAILED: &str = "NEST_AI_REQUEST_FAILED";
/// Response parse failure.
pub const NEST_AI_PARSE_FAILED: &str = "NEST_AI_PARSE_FAILED";
/// Configuration error.
pub const NEST_AI_CONFIG: &str = "NEST_AI_CONFIG";
/// Invalid request parameters.
pub const NEST_AI_INVALID_INPUT: &str = "NEST_AI_INVALID_INPUT";
