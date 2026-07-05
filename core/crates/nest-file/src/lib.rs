//! Sync file I/O for the Nest framework.
//!
//! nest-file provides [`FileService`] for safe, scoped filesystem operations.
//! Format-specific parsers (CSV, JSON, etc.) belong in separate crates.

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

pub mod codes;
mod config;
mod error;
mod metadata;
mod module;
mod path;
pub mod prelude;
pub mod search;
mod service;

pub use config::{FileServiceConfig, WriteOptions};
pub use error::{FileError, FileErrorKind, FileResult};
pub use metadata::{DirEntry, FileMetadata};
pub use module::{FileModule, FILE_MODULE_ID};
pub use path::SafePathResolver;
pub use search::{search_files, FileSearchMatch, FileSearchOptions, DEFAULT_SEARCH_IGNORE};
pub use service::FileService;

pub use nest_core::{Module, ModuleId};
pub use nest_error::{NestError, NestResult};

#[cfg(test)]
mod tests {
    use nest_error::{codes, NestErrorKind};
    use tempfile::tempdir;

    use super::*;
    use crate::config::WriteOptions;

    #[test]
    fn read_write_round_trip() {
        let dir = tempdir().unwrap();
        let files =
            FileService::with_config(FileServiceConfig::scoped(dir.path()).allow_create_dirs(true))
                .unwrap();

        files.write_text("output/report.txt", "hello nest").unwrap();
        let text = files.read_text("output/report.txt").unwrap();
        assert_eq!(text, "hello nest");
    }

    #[test]
    fn atomic_write_replaces_content() {
        let dir = tempdir().unwrap();
        let files =
            FileService::with_config(FileServiceConfig::scoped(dir.path()).allow_create_dirs(true))
                .unwrap();
        files.write_text("data.txt", "original").unwrap();

        let mut options = WriteOptions::from_config(files.config());
        options.atomic = true;
        files
            .write_bytes_with_options("data.txt", b"updated", options)
            .unwrap();

        assert_eq!(files.read_text("data.txt").unwrap(), "updated");
    }

    #[test]
    fn backup_before_write() {
        let dir = tempdir().unwrap();
        let files =
            FileService::with_config(FileServiceConfig::scoped(dir.path()).allow_create_dirs(true))
                .unwrap();
        files.write_text("config.toml", "v1").unwrap();

        let mut options = WriteOptions::from_config(files.config());
        options.backup = true;
        files
            .write_bytes_with_options("config.toml", b"v2", options)
            .unwrap();

        assert_eq!(files.read_text("config.toml").unwrap(), "v2");
        assert_eq!(files.read_text("config.toml.bak").unwrap(), "v1");
    }

    #[test]
    fn copy_move_delete() {
        let dir = tempdir().unwrap();
        let files =
            FileService::with_config(FileServiceConfig::scoped(dir.path()).allow_create_dirs(true))
                .unwrap();
        files.write_text("source.txt", "payload").unwrap();
        files.copy("source.txt", "copy.txt").unwrap();
        assert_eq!(files.read_text("copy.txt").unwrap(), "payload");

        files.move_file("copy.txt", "moved.txt").unwrap();
        assert!(!files.exists("copy.txt").unwrap());
        assert!(files.exists("moved.txt").unwrap());

        files.delete_file("moved.txt").unwrap();
        assert!(!files.exists("moved.txt").unwrap());
    }

    #[test]
    fn list_dir_and_metadata() {
        let dir = tempdir().unwrap();
        let files =
            FileService::with_config(FileServiceConfig::scoped(dir.path()).allow_create_dirs(true))
                .unwrap();
        files.write_text("a.txt", "a").unwrap();
        files.write_text("b.txt", "bb").unwrap();

        let entries = files.list_dir(".").unwrap();
        assert_eq!(entries.len(), 2);

        let meta = files.metadata("b.txt").unwrap();
        assert!(meta.is_file);
        assert_eq!(meta.len, 2);
    }

    #[test]
    fn scoped_rejects_traversal_via_service() {
        let dir = tempdir().unwrap();
        let files =
            FileService::with_config(FileServiceConfig::scoped(dir.path()).allow_create_dirs(true))
                .unwrap();
        let err = files.read_text("../outside.txt").unwrap_err();
        assert_eq!(err.code(), Some(codes::NEST_FILE_PATH_TRAVERSAL_DENIED));
    }

    #[test]
    fn file_error_converts_to_nest_error() {
        let error = FileError::not_found("missing");
        let nest_error: NestError = error.into();
        assert_eq!(nest_error.kind(), NestErrorKind::Io);
        assert_eq!(nest_error.code(), Some(codes::NEST_FILE_NOT_FOUND));
    }
}
