//! Resolves how Nest Shell should launch registered apps (embed, spawn, module).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::apps::{discover_registered_apps, RegisteredApp};
use crate::nest_root::resolve_nest_root;

const BASE_EMBED_PORT: u16 = 5174;
const DEFAULT_BUILD_ENTRY: &str = "ui/dist/index.html";
const PORTS_FILE: &str = "ui/.embed-dev-ports.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LaunchMode {
    Module,
    Embed,
    Spawn,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchTarget {
    pub app_id: String,
    pub mode: LaunchMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dev_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub program: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PortsFile {
    #[serde(flatten)]
    ports: BTreeMap<String, u16>,
}

/// Resolves how the shell should launch a registered app.
#[tauri::command]
pub fn apps_resolve_launch(app_id: String) -> Result<LaunchTarget, String> {
    let root = resolve_nest_root()?;
    let apps = discover_registered_apps(&root)?;
    let app = apps
        .into_iter()
        .find(|entry| entry.id == app_id)
        .ok_or_else(|| format!("unknown app: {app_id}"))?;

    resolve_launch_target(&root, &app)
}

/// Spawns an external program for `spawn` launch mode.
#[tauri::command]
pub fn apps_spawn(program: String, args: Vec<String>, cwd: Option<String>) -> Result<u32, String> {
    spawn_program(&program, &args, cwd.as_deref())
}

/// Launches the system `kiwi-desktop` binary from the local Kiwi project folder.
#[tauri::command]
pub fn apps_launch_kiwi() -> Result<u32, String> {
    let root = resolve_nest_root()?;
    let workdir = kiwi_project_workdir(&root);
    if !workdir.is_dir() {
        return Err(format!(
            "kiwi project folder not found at {}",
            workdir.display()
        ));
    }

    let config = workdir.join("config.toml");
    let mut command = std::process::Command::new("kiwi-desktop");
    command.current_dir(&workdir);
    if config.is_file() {
        command.env("KIWI_CONFIG", &config);
    }

    let child = command
        .spawn()
        .map_err(|error| format!("failed to spawn kiwi-desktop: {error}"))?;

    Ok(child.id())
}

pub fn kiwi_project_workdir(root: &Path) -> PathBuf {
    root.join("apps/kiwi/desktop")
}

fn spawn_program(program: &str, args: &[String], cwd: Option<&str>) -> Result<u32, String> {
    let mut command = std::process::Command::new(program);
    command.args(args);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }

    let child = command
        .spawn()
        .map_err(|error| format!("failed to spawn {program}: {error}"))?;

    Ok(child.id())
}

pub fn resolve_launch_target(root: &Path, app: &RegisteredApp) -> Result<LaunchTarget, String> {
    let app_root = root.join(&app.path);
    let manifest = load_launch_manifest(&app_root.join("nest-app.toml"))?;

    match manifest.mode {
        LaunchMode::Spawn => Ok(LaunchTarget {
            app_id: app.id.clone(),
            mode: LaunchMode::Spawn,
            url: None,
            dev_port: None,
            program: Some(manifest.program),
            args: if manifest.args.is_empty() {
                None
            } else {
                Some(manifest.args)
            },
            message: None,
        }),
        LaunchMode::Module => Ok(LaunchTarget {
            app_id: app.id.clone(),
            mode: LaunchMode::Module,
            url: None,
            dev_port: None,
            program: None,
            args: None,
            message: None,
        }),
        LaunchMode::Embed => resolve_embed_target(root, app, &app_root, manifest),
    }
}

fn resolve_embed_target(
    root: &Path,
    app: &RegisteredApp,
    app_root: &Path,
    manifest: LaunchManifest,
) -> Result<LaunchTarget, String> {
    let ui_root = app_root.join("ui");
    if !ui_root.join("package.json").is_file() {
        return Ok(LaunchTarget {
            app_id: app.id.clone(),
            mode: LaunchMode::Module,
            url: None,
            dev_port: None,
            program: None,
            args: None,
            message: Some(format!(
                "{} is configured for embed launch but has no ui/package.json",
                app.name
            )),
        });
    }

    let embed_ports = embed_dev_ports(root);
    let embed_ids = embed_app_ids(root)?;
    let port = resolve_embed_port(&app.id, manifest.dev_port, &embed_ports, &embed_ids);

    if cfg!(debug_assertions) {
        return Ok(LaunchTarget {
            app_id: app.id.clone(),
            mode: LaunchMode::Embed,
            url: Some(format!("http://localhost:{port}")),
            dev_port: Some(port),
            program: None,
            args: None,
            message: None,
        });
    }

    let entry = app_root.join(manifest.build_entry);
    if !entry.is_file() {
        return Ok(LaunchTarget {
            app_id: app.id.clone(),
            mode: LaunchMode::Embed,
            url: None,
            dev_port: None,
            program: None,
            args: None,
            message: Some(format!(
                "build {} before launching {} in production mode",
                entry.display(),
                app.name
            )),
        });
    }

    Ok(LaunchTarget {
        app_id: app.id.clone(),
        mode: LaunchMode::Embed,
        url: Some(entry.to_string_lossy().into_owned()),
        dev_port: None,
        program: None,
        args: None,
        message: None,
    })
}

#[derive(Debug, Clone)]
struct LaunchManifest {
    mode: LaunchMode,
    dev_port: Option<u16>,
    build_entry: String,
    program: String,
    args: Vec<String>,
}

fn load_launch_manifest(manifest_path: &Path) -> Result<LaunchManifest, String> {
    let text = if manifest_path.is_file() {
        fs::read_to_string(manifest_path)
            .map_err(|error| format!("failed to read {}: {error}", manifest_path.display()))?
    } else {
        String::new()
    };

    let table: toml::Value = if text.trim().is_empty() {
        toml::Value::Table(toml::map::Map::new())
    } else {
        toml::from_str(&text)
            .map_err(|error| format!("failed to parse {}: {error}", manifest_path.display()))?
    };

    let shell = table
        .get("shell")
        .and_then(|value| value.as_table())
        .cloned()
        .unwrap_or_default();
    let launch = shell
        .get("launch")
        .and_then(|value| value.as_table())
        .cloned()
        .unwrap_or_default();
    let dev = shell
        .get("dev")
        .and_then(|value| value.as_table())
        .cloned()
        .unwrap_or_default();
    let build = shell
        .get("build")
        .and_then(|value| value.as_table())
        .cloned()
        .unwrap_or_default();

    let app_root = manifest_path.parent().unwrap_or(Path::new("."));
    let mode = infer_launch_mode(&launch, app_root);
    let dev_port = dev
        .get("port")
        .and_then(|value| value.as_integer())
        .and_then(|value| u16::try_from(value).ok());
    let build_entry = build
        .get("entry")
        .and_then(|value| value.as_str())
        .unwrap_or(DEFAULT_BUILD_ENTRY)
        .to_string();
    let program = launch
        .get("program")
        .and_then(|value| value.as_str())
        .map(str::to_string)
        .unwrap_or_else(|| {
            app_root
                .file_name()
                .and_then(|name| name.to_str())
                .map(|id| format!("{id}-desktop"))
                .unwrap_or_else(|| "app-desktop".into())
        });
    let args = launch
        .get("args")
        .and_then(|value| value.as_array())
        .map(|values| {
            values
                .iter()
                .filter_map(|value| value.as_str().map(str::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(LaunchManifest {
        mode,
        dev_port,
        build_entry,
        program,
        args,
    })
}

fn infer_launch_mode(launch: &toml::map::Map<String, toml::Value>, _app_root: &Path) -> LaunchMode {
    if let Some(mode) = launch
        .get("mode")
        .and_then(|value| value.as_str())
        .map(str::to_ascii_lowercase)
    {
        return match mode.as_str() {
            "embed" => LaunchMode::Embed,
            "spawn" => LaunchMode::Spawn,
            "module" => LaunchMode::Module,
            _ => LaunchMode::Module,
        };
    }

    LaunchMode::Module
}

pub fn embed_app_ids(root: &Path) -> Result<Vec<String>, String> {
    let apps = discover_registered_apps(root)?;
    let mut embed_ids = Vec::new();

    for app in apps {
        let app_root = root.join(&app.path);
        let manifest = load_launch_manifest(&app_root.join("nest-app.toml"))?;
        if manifest.mode == LaunchMode::Embed && app_root.join("ui/package.json").is_file() {
            embed_ids.push(app.id);
        }
    }

    embed_ids.sort();
    Ok(embed_ids)
}

fn embed_dev_ports(root: &Path) -> BTreeMap<String, u16> {
    let ports_path = root.join(PORTS_FILE);
    if !ports_path.is_file() {
        return BTreeMap::new();
    }

    let text = match fs::read_to_string(&ports_path) {
        Ok(text) => text,
        Err(_) => return BTreeMap::new(),
    };

    match serde_json::from_str::<PortsFile>(&text) {
        Ok(file) => file.ports,
        Err(_) => BTreeMap::new(),
    }
}

fn resolve_embed_port(
    app_id: &str,
    manifest_port: Option<u16>,
    ports_file: &BTreeMap<String, u16>,
    embed_ids: &[String],
) -> u16 {
    if let Some(port) = ports_file.get(app_id) {
        return *port;
    }
    if let Some(port) = manifest_port {
        return port;
    }
    if let Some(index) = embed_ids.iter().position(|id| id == app_id) {
        return BASE_EMBED_PORT + index as u16;
    }
    BASE_EMBED_PORT
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn defaults_to_module_when_launch_mode_omitted() {
        let dir = tempdir().unwrap();
        let app = dir.path().join("apps").join("demo");
        fs::create_dir_all(app.join("ui")).unwrap();
        fs::write(app.join("ui/package.json"), "{}").unwrap();
        fs::write(
            app.join("nest-app.toml"),
            r#"
[shell]
name = "Demo"
"#,
        )
        .unwrap();

        let manifest = load_launch_manifest(&app.join("nest-app.toml")).unwrap();
        assert_eq!(manifest.mode, LaunchMode::Module);
    }

    #[test]
    fn respects_spawn_mode_in_manifest() {
        let dir = tempdir().unwrap();
        let app = dir.path().join("apps").join("kiwi");
        fs::create_dir_all(app.join("ui")).unwrap();
        fs::write(app.join("ui/package.json"), "{}").unwrap();
        fs::write(
            app.join("nest-app.toml"),
            r#"
[shell.launch]
mode = "spawn"
program = "kiwi-desktop"
"#,
        )
        .unwrap();

        let manifest = load_launch_manifest(&app.join("nest-app.toml")).unwrap();
        assert_eq!(manifest.mode, LaunchMode::Spawn);
        assert_eq!(manifest.program, "kiwi-desktop");
    }

    #[test]
    fn kiwi_project_workdir_is_under_apps() {
        let dir = tempdir().unwrap();
        let workdir = kiwi_project_workdir(dir.path());
        assert_eq!(workdir, dir.path().join("apps/kiwi/desktop"));
    }

    #[test]
    fn resolves_dev_embed_url_with_assigned_port() {
        let dir = tempdir().unwrap();
        let app_root = dir.path().join("apps").join("swift");
        fs::create_dir_all(app_root.join("ui")).unwrap();
        fs::write(app_root.join("ui/package.json"), "{}").unwrap();
        fs::write(
            app_root.join("nest-app.toml"),
            r#"
[shell]
name = "Swift"

[shell.launch]
mode = "embed"

[shell.dev]
port = 5174
"#,
        )
        .unwrap();

        let app = RegisteredApp {
            id: "swift".into(),
            name: "Swift".into(),
            category: "Development".into(),
            icon: "fa-solid fa-bolt".into(),
            description: String::new(),
            path: "apps/swift".into(),
        };

        let target = resolve_launch_target(dir.path(), &app).unwrap();
        assert_eq!(target.mode, LaunchMode::Embed);
        assert_eq!(target.url.as_deref(), Some("http://localhost:5174"));
    }
}
