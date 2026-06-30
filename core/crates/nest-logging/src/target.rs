//! Log output targets.

/// Where log output is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogTarget {
    /// Standard output (developer console).
    Console,
    /// Rolling text log file in the configured directory.
    File,
    /// Rolling JSON log file in the configured directory.
    JsonFile,
}
