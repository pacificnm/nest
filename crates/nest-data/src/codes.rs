//! Stable data error codes.

/// Generic data operation failure.
pub const NEST_DATA_FAILED: &str = "NEST_DATA_FAILED";

/// Connection id was not registered.
pub const NEST_DATA_CONNECTION_NOT_FOUND: &str = "NEST_DATA_CONNECTION_NOT_FOUND";

/// Connection id was already registered.
pub const NEST_DATA_CONNECTION_ALREADY_REGISTERED: &str = "NEST_DATA_CONNECTION_ALREADY_REGISTERED";

/// No active connection is set.
pub const NEST_DATA_NO_ACTIVE_CONNECTION: &str = "NEST_DATA_NO_ACTIVE_CONNECTION";

/// Migration operation failed.
pub const NEST_DATA_MIGRATION_FAILED: &str = "NEST_DATA_MIGRATION_FAILED";
