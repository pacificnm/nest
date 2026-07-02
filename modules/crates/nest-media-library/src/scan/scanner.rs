//! Recursive filesystem scanner.

use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use nest_file::FileService;
use tracing::{debug, instrument};

use crate::config::MediaLibraryConfig;
use crate::error::{LibraryError, LibraryResult};
use crate::parse::filename::guess_from_path;

use super::models::{MovieScanCandidate, ScanItemStatus, ScanResult, ScannedFile};
use super::stats::{initial_stats, record_candidate, record_error, record_file_seen};

/// Discovers media files in configured library roots.
#[derive(Clone)]
pub struct LibraryScanner {
    files: FileService,
}

impl LibraryScanner {
    /// Creates a scanner backed by the given file service.
    pub fn new(files: FileService) -> Self {
        Self { files }
    }

    /// Walks library roots and returns discovered movie candidates.
    #[instrument(skip(self, config), fields(library_id = %config.id))]
    pub fn discover(&self, config: &MediaLibraryConfig) -> LibraryResult<ScanResult> {
        if config.roots.is_empty() {
            return Err(LibraryError::config("library must have at least one root"));
        }

        let started_at = unix_now();
        let mut result = ScanResult {
            library_id: config.id.clone(),
            started_at,
            finished_at: started_at,
            candidates: Vec::new(),
            errors: Vec::new(),
            stats: initial_stats(),
        };

        for root in &config.roots {
            let root = normalize_relative_path(root);
            if let Err(error) = self.walk_root(config, &root, &mut result) {
                record_error(&mut result, &root, error.message());
            }
        }

        result.finished_at = unix_now();
        debug!(
            files_seen = result.stats.files_seen,
            candidates = result.stats.candidates,
            errors = result.stats.errors,
            "library scan complete"
        );
        Ok(result)
    }

    fn walk_root(
        &self,
        config: &MediaLibraryConfig,
        root: &str,
        result: &mut ScanResult,
    ) -> LibraryResult<()> {
        let metadata = self.files.metadata(root).map_err(LibraryError::from)?;
        if !metadata.is_dir {
            return Err(LibraryError::config(format!(
                "library root is not a directory: {root}"
            )));
        }

        self.walk_dir(config, root, result)
    }

    fn walk_dir(
        &self,
        config: &MediaLibraryConfig,
        dir: &str,
        result: &mut ScanResult,
    ) -> LibraryResult<()> {
        let entries = self.files.list_dir(dir).map_err(LibraryError::from)?;

        for entry in entries {
            let relative_path = join_relative(dir, &entry.name);

            if is_symlink(&entry.path) && !config.follow_symlinks {
                debug!(path = %relative_path, "skipping symlink");
                continue;
            }

            if entry.metadata.is_dir {
                if let Err(error) = self.walk_dir(config, &relative_path, result) {
                    record_error(result, &relative_path, error.message());
                }
                continue;
            }

            if !entry.metadata.is_file {
                continue;
            }

            record_file_seen(&mut result.stats);

            if !is_video_file(&relative_path, &config.video_extensions) {
                continue;
            }

            let guess = guess_from_path(&relative_path);
            let scanned = ScannedFile {
                relative_path: relative_path.clone(),
                size_bytes: entry.metadata.len,
                modified_secs: system_time_to_secs(entry.metadata.modified),
            };

            result.candidates.push(MovieScanCandidate {
                file: scanned,
                guessed_title: guess.title,
                guessed_year: guess.year,
                inspection: None,
                metadata: None,
                status: ScanItemStatus::New,
            });
            record_candidate(&mut result.stats);
        }

        Ok(())
    }
}

fn is_video_file(path: &str, extensions: &[String]) -> bool {
    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| {
            extensions
                .iter()
                .any(|allowed| allowed.eq_ignore_ascii_case(ext))
        })
        .unwrap_or(false)
}

fn is_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
}

fn join_relative(parent: &str, name: &str) -> String {
    if parent == "." || parent.is_empty() {
        name.to_string()
    } else {
        format!("{parent}/{name}")
    }
}

fn normalize_relative_path(path: &str) -> String {
    let path = path.replace('\\', "/");
    if path == "/" || path.is_empty() {
        ".".to_string()
    } else {
        path.trim_start_matches('/')
            .trim_end_matches('/')
            .to_string()
    }
}

fn system_time_to_secs(time: Option<SystemTime>) -> Option<u64> {
    time.and_then(|time| time.duration_since(UNIX_EPOCH).ok())
        .map(|duration| duration.as_secs())
}

fn unix_now() -> u64 {
    system_time_to_secs(Some(SystemTime::now())).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MediaLibraryConfig;
    use nest_file::{FileService, FileServiceConfig};
    use std::fs;
    use tempfile::tempdir;

    fn scoped_files(root: &std::path::Path) -> FileService {
        FileService::with_config(FileServiceConfig::scoped(root).allow_create_dirs(true)).unwrap()
    }

    #[test]
    fn discovers_video_files_recursively() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("Movies/Alien (1979)")).unwrap();
        fs::write(
            dir.path().join("Movies/Alien (1979)/Alien (1979).mkv"),
            b"video",
        )
        .unwrap();
        fs::write(dir.path().join("readme.txt"), b"notes").unwrap();

        let scanner = LibraryScanner::new(scoped_files(dir.path()));
        let config = MediaLibraryConfig::new("main", ["Movies"]);
        let result = scanner.discover(&config).unwrap();

        assert_eq!(result.stats.files_seen, 1);
        assert_eq!(result.candidates.len(), 1);
        assert_eq!(
            result.candidates[0].file.relative_path,
            "Movies/Alien (1979)/Alien (1979).mkv"
        );
        assert_eq!(result.candidates[0].guessed_title.as_deref(), Some("Alien"));
        assert_eq!(result.candidates[0].guessed_year, Some(1979));
    }

    #[test]
    fn filters_by_extension() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("clip.avi"), b"video").unwrap();
        fs::write(dir.path().join("photo.jpg"), b"image").unwrap();

        let scanner = LibraryScanner::new(scoped_files(dir.path()));
        let config = MediaLibraryConfig::new("main", ["."]);
        let result = scanner.discover(&config).unwrap();

        assert_eq!(result.stats.files_seen, 2);
        assert_eq!(result.candidates.len(), 1);
        assert_eq!(result.candidates[0].file.relative_path, "clip.avi");
    }

    #[test]
    fn rejects_empty_roots() {
        let scanner = LibraryScanner::new(scoped_files(tempdir().unwrap().path()));
        let config = MediaLibraryConfig::new("main", std::iter::empty::<String>());
        let error = scanner.discover(&config).unwrap_err();
        assert_eq!(error.kind(), crate::error::LibraryErrorKind::Config);
    }
}
