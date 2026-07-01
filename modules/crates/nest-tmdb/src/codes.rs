//! Stable TMDB error codes.

/// Generic TMDB operation failure.
pub const NEST_TMDB_FAILED: &str = "NEST_TMDB_FAILED";

/// TMDB configuration error.
pub const NEST_TMDB_CONFIG: &str = "NEST_TMDB_CONFIG";

/// TMDB API key missing.
pub const NEST_TMDB_API_KEY_MISSING: &str = "NEST_TMDB_API_KEY_MISSING";

/// TMDB HTTP request failure.
pub const NEST_TMDB_REQUEST_FAILED: &str = "NEST_TMDB_REQUEST_FAILED";

/// TMDB response parse failure.
pub const NEST_TMDB_PARSE_FAILED: &str = "NEST_TMDB_PARSE_FAILED";

/// TMDB resource not found.
pub const NEST_TMDB_NOT_FOUND: &str = "NEST_TMDB_NOT_FOUND";

/// TMDB rate limit exceeded.
pub const NEST_TMDB_RATE_LIMITED: &str = "NEST_TMDB_RATE_LIMITED";

/// TMDB API error response.
pub const NEST_TMDB_API_ERROR: &str = "NEST_TMDB_API_ERROR";
