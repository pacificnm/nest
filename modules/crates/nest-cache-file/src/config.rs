//! File cache configuration.

use std::path::PathBuf;

/// Disk cache settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileCacheConfig {
    /// Root directory for `data/` and `meta/` subfolders.
    pub root: PathBuf,
    /// Optional size cap for future LRU eviction.
    pub max_bytes: Option<u64>,
}

impl FileCacheConfig {
    /// Creates configuration for a cache root directory.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            max_bytes: None,
        }
    }

    /// Sets an optional maximum cache size in bytes.
    pub fn with_max_bytes(mut self, max_bytes: u64) -> Self {
        self.max_bytes = Some(max_bytes);
        self
    }
}
