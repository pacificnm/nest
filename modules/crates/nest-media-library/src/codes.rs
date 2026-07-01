//! Stable media library error codes.

/// Generic library operation failure.
pub const NEST_MEDIA_LIBRARY_FAILED: &str = "NEST_MEDIA_LIBRARY_FAILED";

/// Filesystem scan failure.
pub const NEST_MEDIA_LIBRARY_SCAN_FAILED: &str = "NEST_MEDIA_LIBRARY_SCAN_FAILED";

/// Library configuration error.
pub const NEST_MEDIA_LIBRARY_CONFIG: &str = "NEST_MEDIA_LIBRARY_CONFIG";

/// Metadata provider failure during indexing.
pub const NEST_MEDIA_LIBRARY_PROVIDER_FAILED: &str = "NEST_MEDIA_LIBRARY_PROVIDER_FAILED";

/// Media inspection failure during indexing.
pub const NEST_MEDIA_LIBRARY_INSPECTION_FAILED: &str = "NEST_MEDIA_LIBRARY_INSPECTION_FAILED";

/// Repository failure during indexing.
pub const NEST_MEDIA_LIBRARY_REPOSITORY_FAILED: &str = "NEST_MEDIA_LIBRARY_REPOSITORY_FAILED";

/// Filesystem I/O failure during scanning.
pub const NEST_MEDIA_LIBRARY_IO_FAILED: &str = "NEST_MEDIA_LIBRARY_IO_FAILED";
