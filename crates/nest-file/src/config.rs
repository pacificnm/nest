//! File service configuration.

use std::path::PathBuf;

/// Configuration for [`crate::service::FileService`].
#[derive(Debug, Clone)]
pub struct FileServiceConfig {
    /// Optional root directory for scoped mode.
    pub root: Option<PathBuf>,
    /// Whether absolute paths are allowed when no root is set, or when explicitly enabled.
    pub allow_absolute_paths: bool,
    /// Whether write operations may create missing parent directories by default.
    pub create_parent_dirs: bool,
    /// Whether resolved paths may escape the root via symlinks.
    pub allow_symlink_escape: bool,
}

impl Default for FileServiceConfig {
    fn default() -> Self {
        Self {
            root: None,
            allow_absolute_paths: true,
            create_parent_dirs: false,
            allow_symlink_escape: false,
        }
    }
}

impl FileServiceConfig {
    /// Creates a scoped configuration rooted at the given directory.
    pub fn scoped(root: impl Into<PathBuf>) -> Self {
        Self {
            root: Some(root.into()),
            allow_absolute_paths: false,
            create_parent_dirs: false,
            allow_symlink_escape: false,
        }
    }

    /// Sets the root directory.
    pub fn with_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.root = Some(root.into());
        self.allow_absolute_paths = false;
        self.allow_symlink_escape = false;
        self
    }

    /// Sets whether absolute paths are allowed.
    pub fn allow_absolute_paths(mut self, allow: bool) -> Self {
        self.allow_absolute_paths = allow;
        self
    }

    /// Sets whether parent directories are created on write by default.
    pub fn allow_create_dirs(mut self, allow: bool) -> Self {
        self.create_parent_dirs = allow;
        self
    }
}

/// Per-write operation options.
#[derive(Debug, Clone, Copy)]
pub struct WriteOptions {
    /// Create missing parent directories before writing.
    pub create_parent_dirs: bool,
    /// Write to a temporary file and rename atomically.
    pub atomic: bool,
    /// Copy an existing file to `.bak` before overwriting.
    pub backup: bool,
}

impl WriteOptions {
    /// Creates write options from service defaults.
    pub fn from_config(config: &FileServiceConfig) -> Self {
        Self {
            create_parent_dirs: config.create_parent_dirs,
            atomic: false,
            backup: false,
        }
    }

    /// Enables atomic write (temp file + rename).
    pub fn atomic(mut self) -> Self {
        self.atomic = true;
        self
    }

    /// Enables backup to `.bak` before overwrite.
    pub fn backup(mut self) -> Self {
        self.backup = true;
        self
    }

    /// Enables parent directory creation.
    pub fn create_parents(mut self) -> Self {
        self.create_parent_dirs = true;
        self
    }
}
