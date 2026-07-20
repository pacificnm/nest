//! Stable error codes for nest-auth.

/// Generic auth operation failure.
pub const NEST_AUTH_FAILED: &str = "NEST_AUTH_FAILED";
/// No token stored under the requested key.
pub const NEST_AUTH_NOT_FOUND: &str = "NEST_AUTH_NOT_FOUND";
/// Underlying storage I/O failure (file, keyring, etc.).
pub const NEST_AUTH_IO: &str = "NEST_AUTH_IO";
/// Token serialization/deserialization failure.
pub const NEST_AUTH_SERIALIZE: &str = "NEST_AUTH_SERIALIZE";
