//! Nest repository documentation — list and read Markdown under `docs/`.

use std::fs;
use std::path::Path;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocEntry {
    pub path: String,
    pub name: String,
    pub depth: u32,
}

use crate::nest_root::resolve_nest_root;

/// Lists documentation entries (root README first, then all `docs/**/*.md`).
#[tauri::command]
pub fn docs_list() -> Result<Vec<DocEntry>, String> {
    let root = resolve_nest_root()?;
    let mut paths = Vec::new();

    if root.join("README.md").is_file() {
        paths.push("README.md".into());
    }

    collect_markdown(&root.join("docs"), &root, &mut paths)?;
    paths.sort();

    Ok(paths
        .into_iter()
        .map(|path| DocEntry {
            depth: path.matches('/').count() as u32,
            name: display_name(&path),
            path,
        })
        .collect())
}

/// Reads a Markdown file relative to the Nest repository root.
#[tauri::command]
pub fn docs_read(rel_path: String) -> Result<String, String> {
    let root = resolve_nest_root()?;
    let rel = rel_path.trim().trim_start_matches('/');
    if rel.is_empty() || rel.contains("..") {
        return Err("invalid document path".into());
    }

    let path = root.join(rel);
    if !path.is_file() {
        return Err(format!("document not found: {rel}"));
    }
    if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
        return Err("only .md files can be read".into());
    }

    fs::read_to_string(&path).map_err(|error| format!("failed to read {rel}: {error}"))
}

fn collect_markdown(dir: &Path, root: &Path, paths: &mut Vec<String>) -> Result<(), String> {
    if !dir.is_dir() {
        return Ok(());
    }

    let entries = fs::read_dir(dir).map_err(|error| error.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if path.is_dir() {
            collect_markdown(&path, root, paths)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("md") {
            let rel = path
                .strip_prefix(root)
                .map_err(|_| "path outside repository root".to_string())?;
            paths.push(rel.to_string_lossy().replace('\\', "/"));
        }
    }
    Ok(())
}

fn display_name(path: &str) -> String {
    let file = path.rsplit('/').next().unwrap_or(path);
    if file.eq_ignore_ascii_case("README.md") {
        if let Some(parent) = path.rsplit('/').nth(1) {
            return humanize_segment(parent);
        }
        return "Nest Framework".into();
    }
    humanize_segment(file.strip_suffix(".md").unwrap_or(file))
}

fn humanize_segment(segment: &str) -> String {
    segment
        .split('-')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_name_uses_repo_readme_title() {
        assert_eq!(display_name("README.md"), "Nest Framework");
    }

    #[test]
    fn display_name_uses_parent_for_nested_readme() {
        assert_eq!(display_name("docs/nest-core/README.md"), "Nest Core");
    }
}
