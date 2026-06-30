//! Stable error codes for nest-theme operations.

/// Theme with the given id was not found in the registry.
pub const NEST_THEME_NOT_FOUND: &str = "NEST_THEME_NOT_FOUND";

/// A theme with the same id is already registered.
pub const NEST_THEME_ALREADY_REGISTERED: &str = "NEST_THEME_ALREADY_REGISTERED";

/// No active theme has been set.
pub const NEST_THEME_NO_ACTIVE: &str = "NEST_THEME_NO_ACTIVE";

/// Theme file format could not be determined or is unsupported.
pub const NEST_THEME_FORMAT_UNSUPPORTED: &str = "NEST_THEME_FORMAT_UNSUPPORTED";
