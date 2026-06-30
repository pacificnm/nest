//! Log file retention policy and cleanup.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use nest_error::{NestError, NestResult};

use crate::codes::NEST_LOGGING_DIR_CREATE;

/// Policy for retaining old log files on disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RetentionPolicy {
    /// Do not delete old log files.
    #[default]
    Disabled,
    /// Delete files older than the given number of days.
    Days(u32),
    /// Keep only the newest N log files.
    Files(u32),
}

/// Deletes log files in `directory` matching `app_name` according to `policy`.
///
/// Returns the number of files deleted.
pub fn cleanup_logs(
    directory: &Path,
    app_name: &str,
    policy: RetentionPolicy,
) -> NestResult<usize> {
    match policy {
        RetentionPolicy::Disabled => Ok(0),
        RetentionPolicy::Days(days) => cleanup_by_age(directory, app_name, days),
        RetentionPolicy::Files(max_files) => cleanup_by_count(directory, app_name, max_files),
    }
}

fn log_files(directory: &Path, app_name: &str) -> NestResult<Vec<PathBuf>> {
    if !directory.exists() {
        fs::create_dir_all(directory).map_err(|err| {
            NestError::io(format!("failed to create log directory: {directory:?}"))
                .with_code(NEST_LOGGING_DIR_CREATE)
                .with_source(err)
        })?;
        return Ok(Vec::new());
    }

    let prefix = app_name;
    let mut files = Vec::new();

    for entry in fs::read_dir(directory).map_err(|err| {
        NestError::io(format!("failed to read log directory: {directory:?}"))
            .with_code(NEST_LOGGING_DIR_CREATE)
            .with_source(err)
    })? {
        let entry = entry.map_err(|err| {
            NestError::io("failed to read log directory entry")
                .with_code(NEST_LOGGING_DIR_CREATE)
                .with_source(err)
        })?;
        let path = entry.path();
        if path.is_file()
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(prefix) && name.contains(".log"))
        {
            files.push(path);
        }
    }

    Ok(files)
}

fn cleanup_by_age(directory: &Path, app_name: &str, days: u32) -> NestResult<usize> {
    let cutoff = SystemTime::now()
        .checked_sub(Duration::from_secs(u64::from(days) * 24 * 60 * 60))
        .unwrap_or(SystemTime::UNIX_EPOCH);

    let mut deleted = 0;
    for path in log_files(directory, app_name)? {
        let modified = fs::metadata(&path)
            .and_then(|meta| meta.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);
        if modified < cutoff {
            fs::remove_file(&path).map_err(|err| {
                NestError::io(format!("failed to delete log file: {path:?}"))
                    .with_code(NEST_LOGGING_DIR_CREATE)
                    .with_source(err)
            })?;
            deleted += 1;
        }
    }
    Ok(deleted)
}

fn cleanup_by_count(directory: &Path, app_name: &str, max_files: u32) -> NestResult<usize> {
    let mut files = log_files(directory, app_name)?;
    if files.len() <= max_files as usize {
        return Ok(0);
    }

    files.sort_by_key(|path| {
        fs::metadata(path)
            .and_then(|meta| meta.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH)
    });

    let to_delete = files.len() - max_files as usize;
    let mut deleted = 0;
    for path in files.into_iter().take(to_delete) {
        fs::remove_file(&path).map_err(|err| {
            NestError::io(format!("failed to delete log file: {path:?}"))
                .with_code(NEST_LOGGING_DIR_CREATE)
                .with_source(err)
        })?;
        deleted += 1;
    }
    Ok(deleted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration as StdDuration;

    #[test]
    fn cleanup_by_count_keeps_newest() {
        let dir = tempfile::tempdir().unwrap();
        let app = "kiwi";
        for i in 0..3 {
            let path = dir.path().join(format!("{app}.{i}.log"));
            std::fs::write(&path, "log").unwrap();
            thread::sleep(StdDuration::from_millis(5));
        }

        let deleted = cleanup_logs(dir.path(), app, RetentionPolicy::Files(1)).unwrap();
        assert_eq!(deleted, 2);
        let remaining: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(remaining.len(), 1);
    }
}
