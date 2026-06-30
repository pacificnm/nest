//! Log output format selection.

/// Output format for tracing fmt layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogFormat {
    /// Human-readable multi-line output with colors (developer console).
    #[default]
    Pretty,
    /// Single-line compact output.
    Compact,
    /// Newline-delimited JSON for production logs.
    Json,
}
