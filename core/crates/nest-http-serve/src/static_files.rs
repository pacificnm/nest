//! Static file serving configuration.

use std::path::{Path, PathBuf};

/// Static file serving options.
#[derive(Debug, Clone)]
pub struct StaticFilesConfig {
    root: PathBuf,
}

impl StaticFilesConfig {
    /// Creates a static files config for the given root directory.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Returns the filesystem root.
    pub fn root(&self) -> &Path {
        &self.root
    }
}
