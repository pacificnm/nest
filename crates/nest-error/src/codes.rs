//! Stable error code constants for Nest errors.

/// Service type was not registered.
pub const NEST_SERVICE_NOT_FOUND: &str = "NEST_SERVICE_NOT_FOUND";

/// Service type was already registered.
pub const NEST_SERVICE_ALREADY_REGISTERED: &str = "NEST_SERVICE_ALREADY_REGISTERED";

/// Module configuration failed.
pub const NEST_MODULE_CONFIG_FAILED: &str = "NEST_MODULE_CONFIG_FAILED";

/// Required module dependency was not registered.
pub const NEST_MODULE_DEPENDENCY_MISSING: &str = "NEST_MODULE_DEPENDENCY_MISSING";

/// Lifecycle hook failed.
pub const NEST_LIFECYCLE_FAILED: &str = "NEST_LIFECYCLE_FAILED";

/// Unknown or uncategorized error.
pub const NEST_UNKNOWN: &str = "NEST_UNKNOWN";

/// Data operation failed.
pub const NEST_DATA_FAILED: &str = "NEST_DATA_FAILED";

/// Data connection was not found.
pub const NEST_DATA_CONNECTION_NOT_FOUND: &str = "NEST_DATA_CONNECTION_NOT_FOUND";

/// Data migration failed.
pub const NEST_DATA_MIGRATION_FAILED: &str = "NEST_DATA_MIGRATION_FAILED";

/// HTTP request failed.
pub const NEST_HTTP_REQUEST_FAILED: &str = "NEST_HTTP_REQUEST_FAILED";

/// HTTP request timed out.
pub const NEST_HTTP_TIMEOUT: &str = "NEST_HTTP_TIMEOUT";

/// HTTP response decode failed.
pub const NEST_HTTP_DECODE_FAILED: &str = "NEST_HTTP_DECODE_FAILED";

/// Task spawn failed.
pub const NEST_TASK_SPAWN_FAILED: &str = "NEST_TASK_SPAWN_FAILED";

/// Task was cancelled.
pub const NEST_TASK_CANCELLED: &str = "NEST_TASK_CANCELLED";

/// Task was not found in the registry.
pub const NEST_TASK_NOT_FOUND: &str = "NEST_TASK_NOT_FOUND";

/// Tokio runtime is not available.
pub const NEST_TASK_RUNTIME_MISSING: &str = "NEST_TASK_RUNTIME_MISSING";
