//! Stable transcode / probe error codes.

/// Generic transcode operation failure.
pub const NEST_TRANSCODE_FAILED: &str = "NEST_TRANSCODE_FAILED";

/// Transcode configuration error.
pub const NEST_TRANSCODE_CONFIG: &str = "NEST_TRANSCODE_CONFIG";

/// FFprobe binary not found.
pub const NEST_TRANSCODE_BINARY_NOT_FOUND: &str = "NEST_TRANSCODE_BINARY_NOT_FOUND";

/// FFprobe probe failure.
pub const NEST_TRANSCODE_PROBE_FAILED: &str = "NEST_TRANSCODE_PROBE_FAILED";

/// FFprobe JSON parse failure.
pub const NEST_TRANSCODE_PARSE_FAILED: &str = "NEST_TRANSCODE_PARSE_FAILED";

/// FFprobe probe timeout.
pub const NEST_TRANSCODE_TIMEOUT: &str = "NEST_TRANSCODE_TIMEOUT";

/// Filesystem I/O failure during probing.
pub const NEST_TRANSCODE_IO_FAILED: &str = "NEST_TRANSCODE_IO_FAILED";
