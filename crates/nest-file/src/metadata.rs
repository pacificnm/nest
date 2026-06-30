//! File metadata types.

use std::path::PathBuf;
use std::time::SystemTime;

/// Metadata snapshot for a file or directory.
#[derive(Debug, Clone)]
pub struct FileMetadata {
    /// Path this metadata describes.
    pub path: PathBuf,
    /// Byte length for files.
    pub len: u64,
    /// Whether this is a directory.
    pub is_dir: bool,
    /// Whether this is a regular file.
    pub is_file: bool,
    /// Last modification time when available.
    pub modified: Option<SystemTime>,
}

impl From<(PathBuf, std::fs::Metadata)> for FileMetadata {
    fn from((path, metadata): (PathBuf, std::fs::Metadata)) -> Self {
        Self {
            path,
            len: metadata.len(),
            is_dir: metadata.is_dir(),
            is_file: metadata.is_file(),
            modified: metadata.modified().ok(),
        }
    }
}

/// Entry returned from directory listings.
#[derive(Debug, Clone)]
pub struct DirEntry {
    /// File or directory name.
    pub name: String,
    /// Full resolved path.
    pub path: PathBuf,
    /// Entry metadata.
    pub metadata: FileMetadata,
}
