//! Discovers Nest product apps from `apps/*/nest-app.toml` manifests.

use std::fs;
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

use crate::nest_root::resolve_nest_root;

const MANIFEST_FILE: &str = "nest-app.toml";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisteredApp {
    pub id: String,
    pub name: String,
    pub category: String,
    pub icon: String,
    pub description: String,
    /// Relative path from repository root, e.g. `apps/kiwi`.
    pub path: String,
}

#[derive(Debug, Deserialize)]
struct ManifestFile {
    #[serde(default)]
    shell: ShellSection,
}

#[derive(Debug, Default, Deserialize)]
struct ShellSection {
    #[serde(default)]
    name: Option<String>,
    #[serde(default = "default_category")]
    category: String,
    #[serde(default = "default_icon")]
    icon: String,
    #[serde(default)]
    description: String,
    #[serde(default = "default_visible")]
    visible: bool,
}

fn default_category() -> String {
    "Applications".into()
}

fn default_icon() -> String {
    "fa-solid fa-cube".into()
}

fn default_visible() -> bool {
    true
}

/// Lists product apps registered via `nest-app.toml` under `apps/`.
#[tauri::command]
pub fn apps_list() -> Result<Vec<RegisteredApp>, String> {
    let root = resolve_nest_root()?;
    discover_registered_apps(&root)
}

pub fn discover_registered_apps(root: &Path) -> Result<Vec<RegisteredApp>, String> {
    let apps_dir = root.join("apps");
    if !apps_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut apps = Vec::new();
    let entries = fs::read_dir(&apps_dir).map_err(|error| error.to_string())?;

    for entry in entries {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let Some(id) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if id.starts_with('.') {
            continue;
        }

        let manifest_path = path.join(MANIFEST_FILE);
        if !manifest_path.is_file() {
            continue;
        }

        if let Some(app) = load_manifest(root, id, &manifest_path)? {
            apps.push(app);
        }
    }

    apps.sort_by(|left, right| left.name.to_lowercase().cmp(&right.name.to_lowercase()));
    Ok(apps)
}

fn load_manifest(
    root: &Path,
    id: &str,
    manifest_path: &Path,
) -> Result<Option<RegisteredApp>, String> {
    let text = fs::read_to_string(manifest_path)
        .map_err(|error| format!("failed to read {}: {error}", manifest_path.display()))?;
    let manifest: ManifestFile = toml::from_str(&text)
        .map_err(|error| format!("failed to parse {}: {error}", manifest_path.display()))?;

    if !manifest.shell.visible {
        return Ok(None);
    }

    let name = manifest
        .shell
        .name
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| humanize_id(id));

    let rel_path = format!("apps/{id}");

    Ok(Some(RegisteredApp {
        id: id.into(),
        name,
        category: manifest.shell.category,
        icon: manifest.shell.icon,
        description: manifest.shell.description,
        path: path_relative_to_root(root, &root.join(&rel_path))?,
    }))
}

fn path_relative_to_root(root: &Path, path: &Path) -> Result<String, String> {
    path.strip_prefix(root)
        .map(|rel| rel.to_string_lossy().replace('\\', "/"))
        .map_err(|_| format!("path {} is outside repository root", path.display()))
}

fn humanize_id(id: &str) -> String {
    id.split('-')
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
    use tempfile::tempdir;

    #[test]
    fn loads_manifest_from_apps_directory() {
        let dir = tempdir().unwrap();
        let apps = dir.path().join("apps").join("demo");
        fs::create_dir_all(&apps).unwrap();
        fs::write(
            apps.join(MANIFEST_FILE),
            r#"
[shell]
name = "Demo App"
category = "Examples"
icon = "fa-solid fa-rocket"
description = "Example manifest"
"#,
        )
        .unwrap();

        let manifest = load_manifest(dir.path(), "demo", &apps.join(MANIFEST_FILE))
            .unwrap()
            .expect("app");
        assert_eq!(manifest.id, "demo");
        assert_eq!(manifest.name, "Demo App");
        assert_eq!(manifest.path, "apps/demo");
    }
}
