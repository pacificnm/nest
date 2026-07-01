//! Application identity registered for CLI command handlers.

/// Host application name and version supplied by the CLI bootstrap pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliHostInfo {
    /// Application name.
    pub name: String,
    /// Application version string, when configured by the host.
    pub version: Option<String>,
}
