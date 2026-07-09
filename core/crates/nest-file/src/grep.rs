//! Workspace content search (line-oriented grep).

use serde::Serialize;

use crate::search::{query_tokens, DEFAULT_SEARCH_IGNORE};
use crate::{FileService, NestResult};

/// Default extensions searched by [`grep_files`].
pub const DEFAULT_GREP_EXTENSIONS: &[&str] = &[
    "rs", "toml", "md", "json", "py", "js", "ts", "tsx", "css", "html", "yaml", "yml",
];

/// Maximum file size read during content search.
pub const DEFAULT_MAX_FILE_BYTES: u64 = 512 * 1024;

/// Options for [`grep_files`].
#[derive(Debug, Clone)]
pub struct GrepOptions {
    /// Search terms — every token must appear on the same line (case-insensitive).
    pub query: String,
    /// Directory scope relative to the workspace root.
    pub path: String,
    /// Maximum matches to return.
    pub max_results: usize,
    /// Skipped directory names.
    pub ignore_dirs: Vec<String>,
    /// File extensions to include (without dot). Empty uses [`DEFAULT_GREP_EXTENSIONS`].
    pub extensions: Vec<String>,
    /// Skip files larger than this many bytes.
    pub max_file_bytes: u64,
}

impl Default for GrepOptions {
    fn default() -> Self {
        Self {
            query: String::new(),
            path: ".".into(),
            max_results: 30,
            ignore_dirs: DEFAULT_SEARCH_IGNORE
                .iter()
                .map(|name| (*name).to_string())
                .collect(),
            extensions: DEFAULT_GREP_EXTENSIONS
                .iter()
                .map(|ext| (*ext).to_string())
                .collect(),
            max_file_bytes: DEFAULT_MAX_FILE_BYTES,
        }
    }
}

impl GrepOptions {
    /// Creates options for a query string.
    pub fn for_query(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            ..Self::default()
        }
    }

    /// Limits results and optionally scopes to a subdirectory.
    pub fn with_scope(mut self, path: impl Into<String>, max_results: usize) -> Self {
        self.path = path.into();
        self.max_results = max_results.clamp(1, 200);
        self
    }
}

/// One line matched by [`grep_files`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GrepMatch {
    /// Path relative to the workspace root.
    pub path: String,
    /// 1-based line number.
    pub line: usize,
    /// Trimmed line text (truncated when very long).
    pub snippet: String,
}

/// Finds lines whose text contains every query token.
pub fn grep_files(files: &FileService, options: &GrepOptions) -> NestResult<Vec<GrepMatch>> {
    let tokens = query_tokens(&options.query);
    if tokens.is_empty() {
        return Ok(Vec::new());
    }

    let max_results = options.max_results.clamp(1, 200);
    let extensions = if options.extensions.is_empty() {
        DEFAULT_GREP_EXTENSIONS
            .iter()
            .map(|ext| (*ext).to_string())
            .collect::<Vec<_>>()
    } else {
        options.extensions.clone()
    };

    let mut results = Vec::new();
    let mut ctx = GrepContext {
        files,
        tokens: &tokens,
        ignored: &options.ignore_dirs,
        extensions: &extensions,
        max_file_bytes: options.max_file_bytes,
        results: &mut results,
        max_results,
    };
    walk_dir(&mut ctx, &options.path)?;
    Ok(results)
}

struct GrepContext<'a> {
    files: &'a FileService,
    tokens: &'a [String],
    ignored: &'a [String],
    extensions: &'a [String],
    max_file_bytes: u64,
    results: &'a mut Vec<GrepMatch>,
    max_results: usize,
}

fn walk_dir(ctx: &mut GrepContext<'_>, dir_rel: &str) -> NestResult<()> {
    if ctx.results.len() >= ctx.max_results {
        return Ok(());
    }

    let entries = ctx.files.list_dir(dir_rel)?;
    for entry in entries {
        if ctx.results.len() >= ctx.max_results {
            break;
        }

        let rel_path = if dir_rel == "." {
            entry.name.clone()
        } else {
            format!("{dir_rel}/{}", entry.name)
        };

        if entry.metadata.is_dir {
            if !ctx.ignored.iter().any(|name| name == &entry.name) {
                walk_dir(ctx, &rel_path)?;
            }
            continue;
        }

        if !has_allowed_extension(&rel_path, ctx.extensions) {
            continue;
        }

        if entry.metadata.len > ctx.max_file_bytes {
            continue;
        }

        if let Ok(content) = ctx.files.read_text(&rel_path) {
            search_lines(&rel_path, &content, ctx.tokens, ctx.results, ctx.max_results);
        }
    }

    Ok(())
}

fn has_allowed_extension(path: &str, extensions: &[String]) -> bool {
    let Some(ext) = path.rsplit('.').next() else {
        return false;
    };
    extensions.iter().any(|allowed| allowed.eq_ignore_ascii_case(ext))
}

fn search_lines(
    path: &str,
    content: &str,
    tokens: &[String],
    results: &mut Vec<GrepMatch>,
    max_results: usize,
) {
    for (index, line) in content.lines().enumerate() {
        if results.len() >= max_results {
            break;
        }
        let lower = line.to_ascii_lowercase();
        if tokens.iter().all(|token| lower.contains(token)) {
            results.push(GrepMatch {
                path: path.to_string(),
                line: index + 1,
                snippet: truncate_snippet(line),
            });
        }
    }
}

fn truncate_snippet(line: &str) -> String {
    const LIMIT: usize = 240;
    let trimmed = line.trim();
    if trimmed.len() <= LIMIT {
        trimmed.to_string()
    } else {
        format!("{}…", &trimmed[..LIMIT])
    }
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
    fn finds_matching_lines() {
        let dir = tempdir().unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(
            dir.path().join("src/agent.rs"),
            "pub fn run() {}\npub struct AgentLoop;\n",
        )
        .unwrap();
        let files = scoped_files(dir.path());

        let matches = grep_files(
            &files,
            &GrepOptions::for_query("AgentLoop").with_scope(".", 10),
        )
        .unwrap();

        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].path, "src/agent.rs");
        assert_eq!(matches[0].line, 2);
    }

    #[test]
    fn requires_all_tokens_on_same_line() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("a.rs"), "fn alpha() {}\nfn beta() {}\n").unwrap();
        let files = scoped_files(dir.path());

        let matches = grep_files(
            &files,
            &GrepOptions::for_query("fn beta").with_scope(".", 10),
        )
        .unwrap();
        assert_eq!(matches.len(), 1);

        let none = grep_files(
            &files,
            &GrepOptions::for_query("alpha beta").with_scope(".", 10),
        )
        .unwrap();
        assert!(none.is_empty());
    }
}
