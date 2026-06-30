//! Sync file I/O service.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Instant;

use nest_error::{NestError, NestResult};
use tracing::{debug, info, warn};

use crate::config::{FileServiceConfig, WriteOptions};
use crate::error::{map_io_error, FileError, FileResult};
use crate::metadata::{DirEntry, FileMetadata};
use crate::path::SafePathResolver;

/// Sync file operations with optional scoped root and safe path resolution.
#[derive(Clone)]
pub struct FileService {
    config: FileServiceConfig,
    resolver: SafePathResolver,
}

impl FileService {
    /// Creates a file service with default configuration.
    pub fn new() -> NestResult<Self> {
        Self::with_config(FileServiceConfig::default())
    }

    /// Creates a file service from configuration.
    pub fn with_config(config: FileServiceConfig) -> NestResult<Self> {
        if let Some(root) = &config.root {
            if !root.exists() {
                fs::create_dir_all(root).map_err(|error| {
                    FileError::config(format!("failed to create root: {}", root.display()))
                        .with_source(error)
                })?;
            }
        }

        let resolver = SafePathResolver::new(
            config.root.clone(),
            config.allow_absolute_paths,
            config.allow_symlink_escape,
        );

        Ok(Self { config, resolver })
    }

    /// Sets the root directory (scoped mode).
    pub fn with_root(mut self, root: impl Into<PathBuf>) -> NestResult<Self> {
        self.config = self.config.clone().with_root(root);
        self.resolver = SafePathResolver::new(
            self.config.root.clone(),
            self.config.allow_absolute_paths,
            self.config.allow_symlink_escape,
        );
        Ok(self)
    }

    /// Returns the service configuration.
    pub fn config(&self) -> &FileServiceConfig {
        &self.config
    }

    /// Reads a UTF-8 text file.
    pub fn read_text(&self, path: impl AsRef<Path>) -> NestResult<String> {
        let started = Instant::now();
        let input = path.as_ref();
        let resolved = self.resolver.resolve(input).map_err(NestError::from)?;
        let result = fs::read_to_string(&resolved)
            .map_err(|error| NestError::from(map_io_error(error, input).with_path(input)));
        log_read(input, started, result.is_ok());
        result
    }

    /// Writes UTF-8 text to a file.
    pub fn write_text(
        &self,
        path: impl AsRef<Path>,
        content: impl AsRef<str>,
    ) -> NestResult<()> {
        self.write_bytes(path, content.as_ref().as_bytes())
    }

    /// Appends UTF-8 text to a file.
    pub fn append_text(
        &self,
        path: impl AsRef<Path>,
        content: impl AsRef<str>,
    ) -> NestResult<()> {
        let started = Instant::now();
        let input = path.as_ref();
        let options = WriteOptions::from_config(&self.config);
        let resolved = self
            .resolver
            .resolve_for_write(input, options.create_parent_dirs)
            .map_err(NestError::from)?;

        let result = (|| {
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&resolved)
                .map_err(|error| map_io_error(error, input))?;
            file.write_all(content.as_ref().as_bytes())
                .map_err(|error| FileError::write(error.to_string()).with_source(error))
        })()
        .map_err(|error: FileError| NestError::from(error.with_path(input)));

        log_write(input, content.as_ref().len(), started, result.is_ok());
        result
    }

    /// Reads raw bytes from a file.
    pub fn read_bytes(&self, path: impl AsRef<Path>) -> NestResult<Vec<u8>> {
        let started = Instant::now();
        let input = path.as_ref();
        let resolved = self.resolver.resolve(input).map_err(NestError::from)?;
        let result = fs::read(&resolved)
            .map_err(|error| NestError::from(map_io_error(error, input).with_path(input)));
        log_read(input, started, result.is_ok());
        result
    }

    /// Writes raw bytes to a file.
    pub fn write_bytes(&self, path: impl AsRef<Path>, content: &[u8]) -> NestResult<()> {
        self.write_bytes_with_options(path, content, WriteOptions::from_config(&self.config))
    }

    /// Writes raw bytes with explicit options.
    pub fn write_bytes_with_options(
        &self,
        path: impl AsRef<Path>,
        content: &[u8],
        options: WriteOptions,
    ) -> NestResult<()> {
        let started = Instant::now();
        let input = path.as_ref();
        let resolved = self
            .resolver
            .resolve_for_write(input, options.create_parent_dirs)
            .map_err(NestError::from)?;

        let result = self
            .write_bytes_inner(input, &resolved, content, options)
            .map_err(|error: FileError| NestError::from(error.with_path(input)));

        log_write(input, content.len(), started, result.is_ok());
        result
    }

    fn write_bytes_inner(
        &self,
        input: &Path,
        resolved: &Path,
        content: &[u8],
        options: WriteOptions,
    ) -> FileResult<()> {
        if options.backup && resolved.exists() {
            let backup = backup_path(resolved);
            fs::copy(resolved, &backup).map_err(|error| {
                FileError::write(format!("backup failed: {}", backup.display())).with_source(error)
            })?;
        }

        if options.atomic {
            let file_name = resolved
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("file");
            let temp = resolved.with_file_name(format!("{file_name}.nest-tmp-{}", std::process::id()));
            fs::write(&temp, content).map_err(|error| map_io_error(error, input))?;
            if let Err(error) = fs::rename(&temp, resolved) {
                let _ = fs::remove_file(&temp);
                return Err(
                    FileError::write(format!("atomic rename failed: {}", resolved.display()))
                        .with_source(error),
                );
            }
            return Ok(());
        }

        fs::write(resolved, content).map_err(|error| map_io_error(error, input))?;
        Ok(())
    }

    /// Copies a file.
    pub fn copy(&self, from: impl AsRef<Path>, to: impl AsRef<Path>) -> NestResult<()> {
        let started = Instant::now();
        let from_input = from.as_ref();
        let to_input = to.as_ref();
        let resolved_from = self.resolver.resolve(from_input).map_err(NestError::from)?;
        let resolved_to = self
            .resolver
            .resolve_for_write(to_input, self.config.create_parent_dirs)
            .map_err(NestError::from)?;

        let result = fs::copy(&resolved_from, &resolved_to)
            .map(|_| ())
            .map_err(|error| NestError::from(map_io_error(error, from_input).with_path(from_input)));

        debug!(
            file.from = %from_input.display(),
            file.to = %to_input.display(),
            duration_ms = started.elapsed().as_millis() as u64,
            success = result.is_ok(),
            "file copy"
        );
        result
    }

    /// Moves or renames a file.
    pub fn move_file(&self, from: impl AsRef<Path>, to: impl AsRef<Path>) -> NestResult<()> {
        let from_input = from.as_ref();
        let to_input = to.as_ref();
        let resolved_from = self.resolver.resolve(from_input).map_err(NestError::from)?;
        let resolved_to = self
            .resolver
            .resolve_for_write(to_input, self.config.create_parent_dirs)
            .map_err(NestError::from)?;

        match fs::rename(&resolved_from, &resolved_to) {
            Ok(()) => Ok(()),
            Err(error)
                if error.raw_os_error() == Some(18)
                    || matches!(error.kind(), std::io::ErrorKind::CrossesDevices) =>
            {
                self.copy(from_input, to_input)?;
                self.delete_file(from_input)
            }
            Err(error) => Err(NestError::from(
                map_io_error(error, from_input).with_path(from_input),
            )),
        }
    }

    /// Deletes a file.
    pub fn delete_file(&self, path: impl AsRef<Path>) -> NestResult<()> {
        let started = Instant::now();
        let input = path.as_ref();
        let resolved = self.resolver.resolve(input).map_err(NestError::from)?;
        let result = fs::remove_file(&resolved).map_err(|error| {
            NestError::from(
                FileError::delete(format!("delete failed: {}", input.display()))
                    .with_source(error)
                    .with_path(input),
            )
        });
        debug!(
            file.path = %input.display(),
            duration_ms = started.elapsed().as_millis() as u64,
            success = result.is_ok(),
            "file delete"
        );
        result
    }

    /// Returns whether a path exists.
    pub fn exists(&self, path: impl AsRef<Path>) -> NestResult<bool> {
        let resolved = self
            .resolver
            .resolve(path.as_ref())
            .map_err(NestError::from)?;
        Ok(resolved.exists())
    }

    /// Creates a directory and any missing parents.
    pub fn create_dir_all(&self, path: impl AsRef<Path>) -> NestResult<()> {
        let input = path.as_ref();
        let resolved = self
            .resolver
            .resolve_for_write(input, true)
            .map_err(NestError::from)?;
        fs::create_dir_all(&resolved)
            .map_err(|error| NestError::from(map_io_error(error, input).with_path(input)))
    }

    /// Lists entries in a directory.
    pub fn list_dir(&self, path: impl AsRef<Path>) -> NestResult<Vec<DirEntry>> {
        let input = path.as_ref();
        let resolved = self.resolver.resolve(input).map_err(NestError::from)?;
        let read_dir = fs::read_dir(&resolved)
            .map_err(|error| NestError::from(map_io_error(error, input).with_path(input)))?;

        let mut entries = Vec::new();
        for entry in read_dir {
            let entry = entry
                .map_err(|error| NestError::from(map_io_error(error, input).with_path(input)))?;
            let path = entry.path();
            let metadata = entry
                .metadata()
                .map_err(|error| NestError::from(map_io_error(error, &path).with_path(&path)))?;
            let name = entry.file_name().to_string_lossy().into_owned();
            entries.push(DirEntry {
                name,
                path: path.clone(),
                metadata: FileMetadata::from((path, metadata)),
            });
        }

        Ok(entries)
    }

    /// Returns metadata for a path.
    pub fn metadata(&self, path: impl AsRef<Path>) -> NestResult<FileMetadata> {
        let input = path.as_ref();
        let resolved = self.resolver.resolve(input).map_err(NestError::from)?;
        let metadata = fs::metadata(&resolved)
            .map_err(|error| NestError::from(map_io_error(error, input).with_path(input)))?;
        Ok(FileMetadata::from((resolved, metadata)))
    }
}

impl Default for FileService {
    fn default() -> Self {
        Self::new().expect("default file service")
    }
}

fn backup_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("file");
    path.with_file_name(format!("{file_name}.bak"))
}

fn log_read(path: &Path, started: Instant, success: bool) {
    if success {
        info!(
            file.path = %path.display(),
            duration_ms = started.elapsed().as_millis() as u64,
            "file read"
        );
    } else {
        warn!(
            file.path = %path.display(),
            duration_ms = started.elapsed().as_millis() as u64,
            "file read failed"
        );
    }
}

fn log_write(path: &Path, bytes: usize, started: Instant, success: bool) {
    if success {
        info!(
            file.path = %path.display(),
            file.bytes = bytes,
            duration_ms = started.elapsed().as_millis() as u64,
            "file write"
        );
    } else {
        warn!(
            file.path = %path.display(),
            duration_ms = started.elapsed().as_millis() as u64,
            "file write failed"
        );
    }
}
