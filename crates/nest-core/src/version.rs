//! Version information for the Nest framework.

/// The version of nest-core at compile time.
pub const NEST_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Returns the version of nest-core.
pub fn nest_version() -> &'static str {
    NEST_VERSION
}
