//! Shared workspace file operations used by agent tools and Kiwi commands.

use nest_file::FileService;

use crate::{NestError, NestResult};

/// Writes UTF-8 text to a file (creates or overwrites).
pub fn write_file(files: &FileService, path: &str, content: &str) -> NestResult<String> {
    let path = normalize_workspace_path(path)?;
    files.write_text(&path, content)?;
    Ok(format!("Wrote {} bytes to {path}.", content.len()))
}

/// Creates a new empty file; fails if the path already exists.
pub fn create_file(files: &FileService, path: &str) -> NestResult<String> {
    let path = normalize_workspace_path(path)?;
    if files.exists(&path)? {
        return Err(NestError::validation(format!("file already exists: {path}")));
    }
    write_file(files, &path, "")
}

/// Creates a directory; fails if the path already exists.
pub fn create_directory(files: &FileService, path: &str) -> NestResult<String> {
    let path = normalize_workspace_path(path)?;
    if files.exists(&path)? {
        let message = if files.metadata(&path)?.is_dir {
            format!("directory already exists: {path}")
        } else {
            format!("a file already exists at {path}")
        };
        return Err(NestError::validation(message));
    }
    files.create_dir_all(&path)?;
    Ok(format!("Created directory {path}."))
}

/// Creates a directory when missing; succeeds if it already exists.
pub fn ensure_directory(files: &FileService, path: &str) -> NestResult<String> {
    let path = normalize_workspace_path(path)?;
    if files.exists(&path)? {
        if files.metadata(&path)?.is_file {
            return Err(NestError::validation(format!(
                "a file already exists at {path}"
            )));
        }
        return Ok(format!("Directory {path} already exists."));
    }
    files.create_dir_all(&path)?;
    Ok(format!("Created directory {path}."))
}

/// Joins a directory and file name into a project-relative path.
pub fn join_rel_path(parent_dir: &str, file_name: &str) -> NestResult<String> {
    let name = file_name.trim();
    if name.is_empty() {
        return Err(NestError::validation("file name is required"));
    }
    if name.contains('/') || name.contains('\\') {
        return Err(NestError::validation(
            "file name must not contain path separators",
        ));
    }

    let rel_path = if parent_dir == "." {
        name.to_string()
    } else {
        format!("{parent_dir}/{name}")
    };
    validate_rel_path(&rel_path)?;
    Ok(rel_path)
}

/// Normalizes a project-relative path, collapsing `.` and `..` segments.
pub fn normalize_workspace_path(rel_path: &str) -> NestResult<String> {
    let rel_path = rel_path.trim();
    if rel_path.is_empty() {
        return Ok(".".into());
    }
    if rel_path.contains('\\') {
        return Err(NestError::validation("path must use forward slashes"));
    }

    let mut stack: Vec<&str> = Vec::new();
    for part in rel_path.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                if stack.is_empty() {
                    return Err(NestError::validation(format!(
                        "path `{rel_path}` escapes the workspace root. File tool paths are \
                         relative to the opened project folder, not the Kiwi app directory. \
                         Do not prefix with ../ — call search_files with a query to find the \
                         correct path (for example apps/kiwi/docs/agent/agent.md)."
                    )));
                }
                stack.pop();
            }
            other => stack.push(other),
        }
    }

    Ok(if stack.is_empty() {
        ".".into()
    } else {
        stack.join("/")
    })
}

fn validate_rel_path(rel_path: &str) -> NestResult<()> {
    normalize_workspace_path(rel_path).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use nest_core::AppBuilder;
    use nest_file::{FileModule, FileService};
    use tempfile::tempdir;

    fn scoped_files(root: &std::path::Path) -> FileService {
        AppBuilder::new()
            .module(FileModule::scoped(root))
            .build()
            .unwrap()
            .context
            .service::<FileService>()
            .unwrap()
            .clone()
    }

    #[test]
    fn create_file_writes_empty_file() {
        let dir = tempdir().unwrap();
        let files = scoped_files(dir.path());

        let summary = create_file(&files, "new.txt").unwrap();
        assert!(summary.contains("new.txt"));
        assert_eq!(files.read_text("new.txt").unwrap(), "");
    }

    #[test]
    fn create_file_rejects_existing_path() {
        let dir = tempdir().unwrap();
        let files = scoped_files(dir.path());
        files.write_text("exists.txt", "x").unwrap();

        assert!(create_file(&files, "exists.txt").is_err());
    }

    #[test]
    fn create_directory_writes_empty_dir() {
        let dir = tempdir().unwrap();
        let files = scoped_files(dir.path());

        let summary = create_directory(&files, "src").unwrap();
        assert!(summary.contains("src"));
        assert!(files.metadata("src").unwrap().is_dir);
    }

    #[test]
    fn create_directory_rejects_existing_path() {
        let dir = tempdir().unwrap();
        let files = scoped_files(dir.path());
        files.create_dir_all("exists").unwrap();

        assert!(create_directory(&files, "exists").is_err());
    }

    #[test]
    fn join_rel_path_builds_nested_paths() {
        assert_eq!(join_rel_path(".", "main.rs").unwrap(), "main.rs");
        assert_eq!(
            join_rel_path("src/agent", "mod.rs").unwrap(),
            "src/agent/mod.rs"
        );
    }

    #[test]
    fn normalize_workspace_path_collapses_internal_parent_segments() {
        assert_eq!(
            normalize_workspace_path("src/../lib/foo.rs").unwrap(),
            "lib/foo.rs"
        );
    }

    #[test]
    fn normalize_workspace_path_rejects_escape_with_hint() {
        let error = normalize_workspace_path("../docs/agent/agent.md")
            .unwrap_err()
            .to_string();
        assert!(error.contains("escapes the workspace root"));
        assert!(error.contains("search_files"));
    }
}
