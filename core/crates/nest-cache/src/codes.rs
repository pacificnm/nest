//! Stable cache error codes.

/// Generic cache operation failure.
pub const NEST_CACHE_FAILED: &str = "NEST_CACHE_FAILED";

/// Cache entry was not found.
pub const NEST_CACHE_NOT_FOUND: &str = "NEST_CACHE_NOT_FOUND";

/// Cache entry expired.
pub const NEST_CACHE_EXPIRED: &str = "NEST_CACHE_EXPIRED";

/// Adapter I/O failure.
pub const NEST_CACHE_IO: &str = "NEST_CACHE_IO";

/// Serialization failure.
pub const NEST_CACHE_SERIALIZATION: &str = "NEST_CACHE_SERIALIZATION";

/// Adapter rejected the operation.
pub const NEST_CACHE_ADAPTER: &str = "NEST_CACHE_ADAPTER";
