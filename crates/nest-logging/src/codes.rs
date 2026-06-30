//! Stable error codes for nest-logging operations.

/// EnvFilter directive string was invalid.
pub const NEST_LOGGING_FILTER_INVALID: &str = "NEST_LOGGING_FILTER_INVALID";

/// Failed to create the log directory.
pub const NEST_LOGGING_DIR_CREATE: &str = "NEST_LOGGING_DIR_CREATE";

/// Global tracing subscriber was already initialized.
pub const NEST_LOGGING_ALREADY_INIT: &str = "NEST_LOGGING_ALREADY_INIT";

/// Rotation policy is not supported in v1.
pub const NEST_LOGGING_ROTATION_UNSUPPORTED: &str = "NEST_LOGGING_ROTATION_UNSUPPORTED";

/// No log targets were configured.
pub const NEST_LOGGING_NO_TARGETS: &str = "NEST_LOGGING_NO_TARGETS";

/// Log directory is required for file targets.
pub const NEST_LOGGING_NO_DIRECTORY: &str = "NEST_LOGGING_NO_DIRECTORY";
