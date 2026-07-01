//! SPA serving configuration.

use std::path::{Path, PathBuf};

/// Single-page application serving options.
#[derive(Debug, Clone)]
pub struct SpaConfig {
    dist_dir: PathBuf,
    index_file: PathBuf,
}

impl SpaConfig {
    /// Creates SPA config for a dist directory.
    pub fn new(dist_dir: impl Into<PathBuf>) -> Self {
        let dist_dir = dist_dir.into();
        let index_file = dist_dir.join("index.html");
        Self {
            dist_dir,
            index_file,
        }
    }

    /// Returns the dist directory.
    pub fn dist_dir(&self) -> &Path {
        &self.dist_dir
    }

    /// Returns the index.html path.
    pub fn index_file(&self) -> &Path {
        &self.index_file
    }
}
