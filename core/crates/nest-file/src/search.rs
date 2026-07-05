//! Workspace file path search.

use serde::Serialize;

use crate::{FileService, NestResult};

/// Default directory names skipped during recursive search.
pub const DEFAULT_SEARCH_IGNORE: &[&str] = &[
    ".git",
    "target",
    "node_modules",
    ".venv",
    "dist",
    "build",
];

/// Options for [`search_files`].
#[derive(Debug, Clone)]
pub struct FileSearchOptions {
    /// Substring tokens matched against the relative path (case-insensitive).
    pub query: String,
    /// Directory to search from, relative to the workspace root.
    pub path: String,
    /// Maximum matches to return.
    pub max_results: usize,
    /// Directory entry names that are never descended into.
    pub ignore_dirs: Vec<String>,
}

impl Default for FileSearchOptions {
    fn default() -> Self {
        Self {
            query: String::new(),
            path: ".".into(),
            max_results: 50,
            ignore_dirs: DEFAULT_SEARCH_IGNORE
                .iter()
                .map(|name| (*name).to_string())
                .collect(),
        }
    }
}

impl FileSearchOptions {
    /// Creates search options for a query string.
    pub fn for_query(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            ..Self::default()
        }
    }

    /// Limits results and optionally scopes to a subdirectory.
    pub fn with_scope(mut self, path: impl Into<String>, max_results: usize) -> Self {
        self.path = path.into();
        self.max_results = max_results.clamp(1, 500);
        self
    }
}

/// One path matched by [`search_files`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FileSearchMatch {
    /// Path relative to the workspace root.
    pub path: String,
    /// Whether the match is a directory.
    pub is_dir: bool,
}

/// Finds files and directories whose relative path contains every query token.
pub fn search_files(files: &FileService, options: &FileSearchOptions) -> NestResult<Vec<FileSearchMatch>> {
    let tokens = query_tokens(&options.query);
    if tokens.is_empty() {
        return Ok(Vec::new());
    }

    let max_results = options.max_results.clamp(1, 500);
    let mut results = Vec::new();
    walk_dir(
        files,
        &options.path,
        &tokens,
        &options.ignore_dirs,
        &mut results,
        max_results,
    )?;
    sort_matches(&mut results);
    Ok(results)
}

fn query_tokens(query: &str) -> Vec<String> {
    query
        .split_whitespace()
        .map(|token| token.to_ascii_lowercase())
        .filter(|token| !token.is_empty())
        .collect()
}

fn walk_dir(
    files: &FileService,
    dir_rel: &str,
    tokens: &[String],
    ignored: &[String],
    results: &mut Vec<FileSearchMatch>,
    max_results: usize,
) -> NestResult<()> {
    if results.len() >= max_results {
        return Ok(());
    }

    let entries = files.list_dir(dir_rel)?;
    for entry in entries {
        if results.len() >= max_results {
            break;
        }

        let rel_path = if dir_rel == "." {
            entry.name.clone()
        } else {
            format!("{dir_rel}/{}", entry.name)
        };

        if path_matches(&rel_path, tokens) {
            results.push(FileSearchMatch {
                path: rel_path.clone(),
                is_dir: entry.metadata.is_dir,
            });
        }

        if entry.metadata.is_dir && !should_ignore_dir(&entry.name, ignored) {
            walk_dir(files, &rel_path, tokens, ignored, results, max_results)?;
        }
    }

    Ok(())
}

fn path_matches(path: &str, tokens: &[String]) -> bool {
    let lower = path.to_ascii_lowercase();
    tokens.iter().all(|token| lower.contains(token))
}

fn should_ignore_dir(name: &str, ignored: &[String]) -> bool {
    ignored.iter().any(|entry| entry == name)
}

fn sort_matches(results: &mut [FileSearchMatch]) {
    results.sort_by(|left, right| {
        match (left.is_dir, right.is_dir) {
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            _ => left.path.to_ascii_lowercase().cmp(&right.path.to_ascii_lowercase()),
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FileServiceConfig;
    use std::fs;
    use tempfile::tempdir;

    fn scoped_files(root: &std::path::Path) -> FileService {
        FileService::with_config(FileServiceConfig::scoped(root)).unwrap()
    }

    #[test]
    fn finds_files_by_path_tokens() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src/agent")).unwrap();
        fs::write(dir.path().join("src/agent/mod.rs"), "mod").unwrap();
        fs::write(dir.path().join("Cargo.toml"), "workspace").unwrap();

        let files = scoped_files(dir.path());
        let matches = search_files(
            &files,
            &FileSearchOptions::for_query("agent mod.rs").with_scope(".", 20),
        )
        .unwrap();

        assert!(matches.iter().any(|entry| entry.path == "src/agent/mod.rs"));
        assert!(!matches.iter().any(|entry| entry.path == "Cargo.toml"));
    }

    #[test]
    fn ignores_build_directories() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("target/debug")).unwrap();
        fs::write(dir.path().join("target/debug/app"), "bin").unwrap();
        fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();

        let files = scoped_files(dir.path());
        let matches = search_files(&files, &FileSearchOptions::for_query("app")).unwrap();

        assert!(matches.is_empty());
    }

    #[test]
    fn empty_query_returns_no_results() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
        let files = scoped_files(dir.path());
        let matches = search_files(&files, &FileSearchOptions::for_query("   ")).unwrap();
        assert!(matches.is_empty());
    }
}
