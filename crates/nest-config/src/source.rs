//! Configuration source kinds.

use std::path::PathBuf;

use crate::document::ConfigDocument;

/// Where configuration is loaded from.
#[derive(Debug, Clone)]
pub enum ConfigSource {
    /// Load from an explicit file path (must exist).
    File(PathBuf),
    /// Search default paths for the application name.
    SearchDefaults,
    /// Pre-built in-memory document.
    Memory(ConfigDocument),
}

impl ConfigSource {
    /// Returns a short label for logging.
    pub fn label(&self) -> &'static str {
        match self {
            Self::File(_) => "file",
            Self::SearchDefaults => "search_defaults",
            Self::Memory(_) => "memory",
        }
    }
}
