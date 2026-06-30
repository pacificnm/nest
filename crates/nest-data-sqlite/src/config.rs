//! SQLite connection configuration.

use std::path::{Path, PathBuf};

/// SQLite connection options.
#[derive(Debug, Clone)]
pub struct SqliteConfig {
    /// Database file path or `:memory:`.
    pub path: PathBuf,
    /// Optional pragmas applied on open.
    pub pragmas: Vec<(String, String)>,
}

impl SqliteConfig {
    /// Opens an in-memory database.
    pub fn memory() -> Self {
        Self::file(":memory:")
    }

    /// Opens a database at the given path.
    pub fn file(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            pragmas: Vec::new(),
        }
    }

    /// Adds a pragma applied after connect.
    pub fn with_pragma(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.pragmas.push((name.into(), value.into()));
        self
    }

    /// Returns the datasource string for connection metadata.
    pub fn datasource(&self) -> String {
        self.path.to_string_lossy().into_owned()
    }
}
