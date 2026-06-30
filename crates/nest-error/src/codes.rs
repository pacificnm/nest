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

/// File or directory was not found.
pub const NEST_FILE_NOT_FOUND: &str = "NEST_FILE_NOT_FOUND";

/// File permission denied.
pub const NEST_FILE_PERMISSION_DENIED: &str = "NEST_FILE_PERMISSION_DENIED";

/// File read failed.
pub const NEST_FILE_READ_FAILED: &str = "NEST_FILE_READ_FAILED";

/// File write failed.
pub const NEST_FILE_WRITE_FAILED: &str = "NEST_FILE_WRITE_FAILED";

/// File delete failed.
pub const NEST_FILE_DELETE_FAILED: &str = "NEST_FILE_DELETE_FAILED";

/// Empty file path.
pub const NEST_FILE_EMPTY_PATH: &str = "NEST_FILE_EMPTY_PATH";

/// Absolute path denied in scoped mode.
pub const NEST_FILE_ABSOLUTE_PATH_DENIED: &str = "NEST_FILE_ABSOLUTE_PATH_DENIED";

/// Path traversal denied.
pub const NEST_FILE_PATH_TRAVERSAL_DENIED: &str = "NEST_FILE_PATH_TRAVERSAL_DENIED";

/// Resolved path is outside configured root.
pub const NEST_FILE_PATH_OUTSIDE_ROOT: &str = "NEST_FILE_PATH_OUTSIDE_ROOT";

/// Parent directory not found.
pub const NEST_FILE_PARENT_NOT_FOUND: &str = "NEST_FILE_PARENT_NOT_FOUND";

/// Symlink escape denied.
pub const NEST_FILE_SYMLINK_ESCAPE_DENIED: &str = "NEST_FILE_SYMLINK_ESCAPE_DENIED";

/// CSV parse failed.
pub const NEST_CSV_PARSE_FAILED: &str = "NEST_CSV_PARSE_FAILED";

/// CSV row deserialize failed.
pub const NEST_CSV_DESERIALIZE_FAILED: &str = "NEST_CSV_DESERIALIZE_FAILED";

/// Required CSV column missing.
pub const NEST_CSV_REQUIRED_COLUMN_MISSING: &str = "NEST_CSV_REQUIRED_COLUMN_MISSING";

/// CSV row validation failed.
pub const NEST_CSV_VALIDATION_FAILED: &str = "NEST_CSV_VALIDATION_FAILED";

/// CSV write failed.
pub const NEST_CSV_WRITE_FAILED: &str = "NEST_CSV_WRITE_FAILED";

/// CSV error limit exceeded.
pub const NEST_CSV_ROW_LIMIT_EXCEEDED: &str = "NEST_CSV_ROW_LIMIT_EXCEEDED";

/// Configuration file not found.
pub const NEST_CONFIG_NOT_FOUND: &str = "NEST_CONFIG_NOT_FOUND";

/// Configuration file parse failed.
pub const NEST_CONFIG_PARSE_FAILED: &str = "NEST_CONFIG_PARSE_FAILED";

/// Configuration file read failed.
pub const NEST_CONFIG_READ_FAILED: &str = "NEST_CONFIG_READ_FAILED";

/// Configuration section missing.
pub const NEST_CONFIG_SECTION_MISSING: &str = "NEST_CONFIG_SECTION_MISSING";

/// Configuration section invalid.
pub const NEST_CONFIG_SECTION_INVALID: &str = "NEST_CONFIG_SECTION_INVALID";

/// Configuration format unsupported.
pub const NEST_CONFIG_UNSUPPORTED_FORMAT: &str = "NEST_CONFIG_UNSUPPORTED_FORMAT";

/// CLI usage error (invalid arguments or subcommand).
pub const NEST_CLI_USAGE: &str = "NEST_CLI_USAGE";

/// TUI terminal initialization failed.
pub const NEST_TUI_TERMINAL_INIT_FAILED: &str = "NEST_TUI_TERMINAL_INIT_FAILED";

/// TUI event loop failed.
pub const NEST_TUI_EVENT_LOOP_FAILED: &str = "NEST_TUI_EVENT_LOOP_FAILED";

/// GUI eframe startup failed.
pub const NEST_GUI_EFRAME_START_FAILED: &str = "NEST_GUI_EFRAME_START_FAILED";

/// Application container was already started.
pub const NEST_APP_ALREADY_STARTED: &str = "NEST_APP_ALREADY_STARTED";

/// Application container was not started.
pub const NEST_APP_NOT_STARTED: &str = "NEST_APP_NOT_STARTED";
