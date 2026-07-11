//! Scoped file browser backend for the Nest Files app.
//!
//! Lists, creates, renames, deletes, and copies files/directories within the
//! Nest repository root, plus "reveal in OS file manager". This is desktop
//! file-manager browsing, not an IDE explorer — no text editing, no search.
//! Ported from Kiwi's `workspace.rs`, trimmed to just the filesystem surface.
//!
//! Mirrors `docs.rs` / `apps.rs`: no managed state, each command resolves the
//! repository root and opens a fresh (stateless) [`FileService`] per call.

use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use nest_file::{FileService, FileServiceConfig};
use serde::Serialize;

use crate::nest_root::resolve_nest_root;

/// Directory names hidden from listings (build artifacts, VCS internals).
const DEFAULT_IGNORE: &[&str] = &[".git", "target", "node_modules", ".venv", "dist", "build"];

/// Largest file the editor will read over IPC (2 MiB).
const MAX_READ_BYTES: u64 = 2 * 1024 * 1024;

/// Root metadata sent to the UI on startup.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FilesRoot {
    /// Absolute repository root on disk.
    pub root: String,
    /// Short display name (repo folder name).
    pub name: String,
}

/// One entry in a directory listing.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    /// File or directory name.
    pub name: String,
    /// Path relative to the repository root (`/`-separated).
    pub rel_path: String,
    /// Whether the entry is a directory.
    pub is_dir: bool,
    /// Byte length (0 for directories).
    pub size: u64,
    /// Last modified time, milliseconds since Unix epoch, when available.
    pub modified: Option<i64>,
}

/// Text contents of a file for the editor.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileContent {
    /// Path relative to the repository root.
    pub rel_path: String,
    /// UTF-8 file contents.
    pub content: String,
}

/// Opens a scoped [`FileService`] rooted at the Nest repository.
fn open() -> Result<(PathBuf, FileService), String> {
    let root = resolve_nest_root()?
        .canonicalize()
        .map_err(|error| format!("invalid Nest repository root: {error}"))?;
    let files = FileService::with_config(FileServiceConfig::scoped(&root))
        .map_err(|error| error.to_string())?;
    Ok((root, files))
}

/// Returns the repository root metadata.
#[tauri::command]
pub fn files_info() -> Result<FilesRoot, String> {
    let (root, _files) = open()?;
    let name = root
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("nest")
        .to_string();
    Ok(FilesRoot {
        root: root.display().to_string(),
        name,
    })
}

/// Lists a directory relative to the repository root, dirs first then names.
///
/// `rel` is `"."` for the root.
#[tauri::command]
pub fn files_list(rel: String) -> Result<Vec<FileEntry>, String> {
    let (_root, files) = open()?;
    let rel = normalize_rel(&rel);
    let mut entries: Vec<FileEntry> = files
        .list_dir(&rel)
        .map_err(|error| error.to_string())?
        .into_iter()
        .filter(|entry| !is_ignored(&entry.name))
        .map(|entry| {
            let rel_path = if rel == "." {
                entry.name.clone()
            } else {
                format!("{rel}/{}", entry.name)
            };
            FileEntry {
                name: entry.name,
                rel_path,
                is_dir: entry.metadata.is_dir,
                size: entry.metadata.len,
                modified: entry.metadata.modified.and_then(|time| {
                    time.duration_since(UNIX_EPOCH)
                        .ok()
                        .map(|duration| duration.as_millis() as i64)
                }),
            }
        })
        .collect();

    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });
    Ok(entries)
}

/// Reads a UTF-8 text file relative to the repository root, for the editor.
///
/// Rejects directories, oversized files, and binary content so the editor
/// never tries to render garbage.
#[tauri::command]
pub fn files_read_text(rel: String) -> Result<FileContent, String> {
    let (_root, files) = open()?;
    let rel = normalize_rel(&rel);
    if rel == "." {
        return Err("cannot open the repository root as a file".to_string());
    }

    let metadata = files.metadata(&rel).map_err(|error| error.to_string())?;
    if metadata.is_dir {
        return Err(format!("{rel} is a directory"));
    }
    if metadata.len > MAX_READ_BYTES {
        return Err(format!(
            "{rel} is too large to open ({} KiB, limit {} KiB)",
            metadata.len / 1024,
            MAX_READ_BYTES / 1024
        ));
    }

    let bytes = files.read_bytes(&rel).map_err(|error| error.to_string())?;
    if bytes.contains(&0) {
        return Err(format!("{rel} looks like a binary file"));
    }
    let content = String::from_utf8(bytes).map_err(|_| format!("{rel} is not valid UTF-8"))?;

    Ok(FileContent {
        rel_path: rel,
        content,
    })
}

/// Writes UTF-8 `content` to an existing (or new) file relative to the
/// repository root. Used by the editor's save action.
#[tauri::command]
pub fn files_write_text(rel: String, content: String) -> Result<String, String> {
    let (_root, files) = open()?;
    let rel = normalize_rel(&rel);
    reject_root(&rel, "save")?;
    if files.exists(&rel).map_err(|error| error.to_string())? {
        let metadata = files.metadata(&rel).map_err(|error| error.to_string())?;
        if metadata.is_dir {
            return Err(format!("{rel} is a directory"));
        }
    }
    files
        .write_text(&rel, &content)
        .map_err(|error| error.to_string())?;
    Ok(rel)
}

/// Creates an empty file at `rel`. Errors if it already exists.
#[tauri::command]
pub fn files_create_file(rel: String) -> Result<String, String> {
    let (_root, files) = open()?;
    let rel = normalize_rel(&rel);
    reject_root(&rel, "create")?;
    if files.exists(&rel).map_err(|error| error.to_string())? {
        return Err(format!("already exists: {rel}"));
    }
    files
        .write_text(&rel, "")
        .map_err(|error| error.to_string())?;
    Ok(rel)
}

/// Creates a directory (and any missing parents) at `rel`.
#[tauri::command]
pub fn files_create_dir(rel: String) -> Result<String, String> {
    let (_root, files) = open()?;
    let rel = normalize_rel(&rel);
    reject_root(&rel, "create")?;
    if files.exists(&rel).map_err(|error| error.to_string())? {
        return Err(format!("already exists: {rel}"));
    }
    files
        .create_dir_all(&rel)
        .map_err(|error| error.to_string())?;
    Ok(rel)
}

/// Renames / moves `from` to `to` (both repo-relative).
#[tauri::command]
pub fn files_rename(from: String, to: String) -> Result<String, String> {
    let (_root, files) = open()?;
    let from = normalize_rel(&from);
    let to = normalize_rel(&to);
    reject_root(&from, "rename")?;
    reject_root(&to, "rename")?;
    if !files.exists(&from).map_err(|error| error.to_string())? {
        return Err(format!("does not exist: {from}"));
    }
    if files.exists(&to).map_err(|error| error.to_string())? {
        return Err(format!("already exists: {to}"));
    }
    files
        .move_file(&from, &to)
        .map_err(|error| error.to_string())?;
    Ok(to)
}

/// Deletes a file or directory tree at `rel`.
#[tauri::command]
pub fn files_delete(rel: String) -> Result<String, String> {
    let (_root, files) = open()?;
    let rel = normalize_rel(&rel);
    reject_root(&rel, "delete")?;
    if !files.exists(&rel).map_err(|error| error.to_string())? {
        return Err(format!("does not exist: {rel}"));
    }
    let metadata = files.metadata(&rel).map_err(|error| error.to_string())?;
    if metadata.is_dir {
        files
            .delete_dir(&rel, true)
            .map_err(|error| error.to_string())?;
    } else {
        files.delete_file(&rel).map_err(|error| error.to_string())?;
    }
    Ok(rel)
}

/// Copies a file or directory tree from `from` to `to`.
#[tauri::command]
pub fn files_copy(from: String, to: String) -> Result<String, String> {
    let (_root, files) = open()?;
    let from = normalize_rel(&from);
    let to = normalize_rel(&to);
    reject_root(&from, "copy")?;
    reject_root(&to, "copy")?;
    if !files.exists(&from).map_err(|error| error.to_string())? {
        return Err(format!("does not exist: {from}"));
    }
    if files.exists(&to).map_err(|error| error.to_string())? {
        return Err(format!("already exists: {to}"));
    }
    let metadata = files.metadata(&from).map_err(|error| error.to_string())?;
    if metadata.is_dir {
        copy_dir_tree(&files, &from, &to)?;
    } else {
        files.copy(&from, &to).map_err(|error| error.to_string())?;
    }
    Ok(to)
}

/// Reveals a path in the OS file manager.
#[tauri::command]
pub fn files_reveal(rel: String) -> Result<(), String> {
    let (root, _files) = open()?;
    let rel = normalize_rel(&rel);
    let abs = if rel == "." { root } else { root.join(&rel) };
    if !abs.exists() {
        return Err(format!("does not exist: {rel}"));
    }
    reveal_in_file_manager(&abs)
}

/// Collapses empty / `"./"` prefixes to the canonical relative form.
fn normalize_rel(rel: &str) -> String {
    let trimmed = rel.trim().trim_start_matches("./").trim_matches('/');
    if trimmed.is_empty() {
        ".".to_string()
    } else {
        trimmed.replace('\\', "/")
    }
}

fn is_ignored(name: &str) -> bool {
    DEFAULT_IGNORE.contains(&name)
}

/// Rejects operating on the repository root itself.
fn reject_root(rel: &str, verb: &str) -> Result<(), String> {
    if rel == "." {
        return Err(format!("cannot {verb} the repository root"));
    }
    Ok(())
}

/// Recursively copies a directory `from` -> `to` (both repo-relative).
fn copy_dir_tree(files: &FileService, from: &str, to: &str) -> Result<(), String> {
    files
        .create_dir_all(to)
        .map_err(|error| error.to_string())?;
    for entry in files.list_dir(from).map_err(|error| error.to_string())? {
        let from_child = format!("{from}/{}", entry.name);
        let to_child = format!("{to}/{}", entry.name);
        if entry.metadata.is_dir {
            copy_dir_tree(files, &from_child, &to_child)?;
        } else if entry.metadata.is_file {
            files
                .copy(&from_child, &to_child)
                .map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

/// Opens the OS file manager focused on `abs` (best-effort, platform-specific).
fn reveal_in_file_manager(abs: &Path) -> Result<(), String> {
    use std::process::{Command, Stdio};

    let spawn = |mut command: Command| -> Result<(), String> {
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("failed to open file manager: {error}"))
    };

    #[cfg(target_os = "macos")]
    {
        let mut command = Command::new("open");
        if abs.is_dir() {
            command.arg(abs);
        } else {
            command.arg("-R").arg(abs);
        }
        return spawn(command);
    }

    #[cfg(target_os = "windows")]
    {
        let mut command = Command::new("explorer");
        if abs.is_dir() {
            command.arg(abs);
        } else {
            command.arg(format!("/select,{}", abs.display()));
        }
        return spawn(command);
    }

    #[cfg(target_os = "linux")]
    {
        let folder = if abs.is_dir() {
            abs.to_path_buf()
        } else {
            abs.parent().unwrap_or(abs).to_path_buf()
        };
        let mut command = Command::new("xdg-open");
        command.arg(&folder);
        spawn(command)
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        let _ = abs;
        Err("open containing folder is not supported on this platform".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn open_at(root: &Path) -> (PathBuf, FileService) {
        let root = root.canonicalize().unwrap();
        let files = FileService::with_config(FileServiceConfig::scoped(&root)).unwrap();
        (root, files)
    }

    fn list_at(root: &Path, rel: &str) -> Vec<FileEntry> {
        let (_root, files) = open_at(root);
        let rel = normalize_rel(rel);
        let mut entries: Vec<FileEntry> = files
            .list_dir(&rel)
            .unwrap()
            .into_iter()
            .filter(|entry| !is_ignored(&entry.name))
            .map(|entry| FileEntry {
                name: entry.name.clone(),
                rel_path: entry.name,
                is_dir: entry.metadata.is_dir,
                size: entry.metadata.len,
                modified: entry.metadata.modified.and_then(|time| {
                    time.duration_since(UNIX_EPOCH)
                        .ok()
                        .map(|duration| duration.as_millis() as i64)
                }),
            })
            .collect();
        entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });
        entries
    }

    #[test]
    fn list_hides_ignored_and_sorts_dirs_first() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::create_dir_all(dir.path().join("target")).unwrap();
        fs::write(dir.path().join("Cargo.toml"), "workspace").unwrap();

        let names: Vec<_> = list_at(dir.path(), ".")
            .into_iter()
            .map(|entry| entry.name)
            .collect();
        assert_eq!(names, vec!["src", "Cargo.toml"]);
    }

    #[test]
    fn list_reports_size_and_modified() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "hello").unwrap();

        let entries = list_at(dir.path(), ".");
        let entry = entries.iter().find(|entry| entry.name == "a.txt").unwrap();
        assert_eq!(entry.size, 5);
        assert!(entry.modified.is_some());
    }

    #[test]
    fn create_file_and_dir() {
        let dir = tempdir().unwrap();
        let (_root, files) = open_at(dir.path());

        files.create_dir_all("src").unwrap();
        assert!(dir.path().join("src").is_dir());
        files.write_text("src/main.rs", "").unwrap();
        assert!(dir.path().join("src/main.rs").is_file());
    }

    #[test]
    fn rename_moves_entry() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.txt"), "x").unwrap();
        let (_root, files) = open_at(dir.path());

        files.move_file("a.txt", "b.txt").unwrap();
        assert!(!dir.path().join("a.txt").exists());
        assert!(dir.path().join("b.txt").is_file());
    }

    #[test]
    fn copy_then_delete_directory_tree() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src/sub")).unwrap();
        fs::write(dir.path().join("src/sub/f.txt"), "y").unwrap();
        let (_root, files) = open_at(dir.path());

        copy_dir_tree(&files, "src", "src-copy").unwrap();
        assert_eq!(
            fs::read_to_string(dir.path().join("src-copy/sub/f.txt")).unwrap(),
            "y"
        );
        assert!(dir.path().join("src/sub/f.txt").exists());

        files.delete_dir("src-copy", true).unwrap();
        assert!(!dir.path().join("src-copy").exists());
    }

    #[test]
    fn reject_root_blocks_root_operations() {
        assert!(reject_root(".", "delete").is_err());
        assert!(reject_root("src/main.rs", "delete").is_ok());
    }
}
