//! Agent settings — load / save from Nest Desktop's `config.toml`.
//!
//! Used by the Settings app (Agent category) to persist connection mode,
//! endpoint, and model list; the Agent app reads the same file to launch.
//! Ported from Kiwi's `agent_config.rs`.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use toml::Value;

/// Agent settings persisted in `[ai]` + `[agent]` config sections.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentSettings {
    /// Ollama inference host (`192.168.88.10` or `server.lan`).
    pub host: String,
    /// Ollama HTTP port.
    pub port: u16,
    /// Active model tag for `ollama launch --model`.
    pub model: String,
    /// Saved model list (merged with live `ollama list` in the UI).
    pub models: Vec<String>,
    /// Default `ollama launch` integration (`claude`, `codex`, …).
    pub runtime: String,
    /// Connection mode: `"ollama"` (server) or `"account"` (native login).
    pub connection: String,
}

impl Default for AgentSettings {
    fn default() -> Self {
        Self {
            host: "192.168.88.10".into(),
            port: 11434,
            model: "qwen3.5:2b".into(),
            models: vec!["qwen3.5:2b".into(), "qwen2.5-coder:7b".into()],
            runtime: "claude".into(),
            connection: "ollama".into(),
        }
    }
}

impl AgentSettings {
    /// `host:port` string used by `ollama list` / `OLLAMA_HOST`.
    #[allow(dead_code)]
    pub fn ollama_host(&self) -> String {
        format!("{}:{}", self.host.trim(), self.port)
    }

    /// HTTP base URL for display.
    pub fn base_url(&self) -> String {
        format!("http://{}:{}", self.host.trim(), self.port)
    }
}

/// Managed Tauri state for Agent settings.
pub struct AgentSettingsStore {
    path: PathBuf,
    settings: Mutex<AgentSettings>,
}

impl AgentSettingsStore {
    /// Loads settings from `path`, or defaults when missing.
    pub fn load(path: PathBuf) -> Self {
        let settings = load_from_path(&path).unwrap_or_default();
        Self {
            path,
            settings: Mutex::new(settings),
        }
    }

    /// Returns a clone of the current settings.
    pub fn get(&self) -> AgentSettings {
        self.settings.lock().expect("agent settings mutex").clone()
    }

    /// Persists `settings` to disk and updates in-memory state.
    pub fn save(&self, settings: AgentSettings) -> Result<AgentSettings, String> {
        save_to_path(&self.path, &settings)?;
        *self.settings.lock().expect("agent settings mutex") = settings.clone();
        Ok(settings)
    }
}

fn load_from_path(path: &Path) -> Option<AgentSettings> {
    let text = fs::read_to_string(path).ok()?;
    let root: Value = text.parse().ok()?;
    let mut settings = AgentSettings::default();

    if let Some(ai) = root.get("ai").and_then(Value::as_table) {
        if let Some(host) = ai.get("host").and_then(Value::as_str) {
            if !host.trim().is_empty() {
                settings.host = host.trim().to_string();
            }
        }
        if let Some(port) = ai.get("port").and_then(Value::as_integer) {
            if (1..=65535).contains(&port) {
                settings.port = port as u16;
            }
        }
        if let Some(model) = ai.get("model").and_then(Value::as_str) {
            if !model.trim().is_empty() {
                settings.model = model.trim().to_string();
            }
        }
        if let Some(models) = ai.get("models").and_then(Value::as_array) {
            let parsed: Vec<String> = models
                .iter()
                .filter_map(|value| value.as_str())
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map(str::to_string)
                .collect();
            if !parsed.is_empty() {
                settings.models = parsed;
            }
        }
    }

    if let Some(agent) = root.get("agent").and_then(Value::as_table) {
        if let Some(model) = agent.get("model").and_then(Value::as_str) {
            if !model.trim().is_empty() {
                settings.model = model.trim().to_string();
            }
        }
        if let Some(runtime) = agent.get("runtime").and_then(Value::as_str) {
            if !runtime.trim().is_empty() {
                settings.runtime = runtime.trim().to_string();
            }
        }
        if let Some(connection) = agent.get("connection").and_then(Value::as_str) {
            if connection == "account" || connection == "ollama" {
                settings.connection = connection.to_string();
            }
        }
    }

    if !settings.models.iter().any(|name| name == &settings.model) {
        settings.models.insert(0, settings.model.clone());
    }

    Some(settings)
}

fn save_to_path(path: &Path, settings: &AgentSettings) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create {}: {error}", parent.display()))?;
    }

    let mut root: Value = if path.is_file() {
        let content = fs::read_to_string(path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
        content
            .parse()
            .map_err(|error| format!("failed to parse {}: {error}", path.display()))?
    } else {
        Value::Table(toml::map::Map::new())
    };

    let table = root
        .as_table_mut()
        .ok_or_else(|| "config root must be a table".to_string())?;

    let ai = table
        .entry("ai")
        .or_insert_with(|| Value::Table(toml::map::Map::new()));
    let ai_table = ai
        .as_table_mut()
        .ok_or_else(|| "[ai] must be a table".to_string())?;
    ai_table.insert("enabled".into(), Value::Boolean(true));
    ai_table.insert("provider".into(), Value::String("ollama".into()));
    ai_table.insert("host".into(), Value::String(settings.host.clone()));
    ai_table.insert("port".into(), Value::Integer(i64::from(settings.port)));
    ai_table.insert("base_url".into(), Value::String(settings.base_url()));
    ai_table.insert("model".into(), Value::String(settings.model.clone()));
    ai_table.insert(
        "models".into(),
        Value::Array(
            settings
                .models
                .iter()
                .map(|name| Value::String(name.clone()))
                .collect(),
        ),
    );

    let agent = table
        .entry("agent")
        .or_insert_with(|| Value::Table(toml::map::Map::new()));
    let agent_table = agent
        .as_table_mut()
        .ok_or_else(|| "[agent] must be a table".to_string())?;
    agent_table.insert("model".into(), Value::String(settings.model.clone()));
    agent_table.insert("runtime".into(), Value::String(settings.runtime.clone()));
    agent_table.insert(
        "connection".into(),
        Value::String(settings.connection.clone()),
    );

    let serialized = toml::to_string_pretty(&root)
        .map_err(|error| format!("failed to serialize config: {error}"))?;
    fs::write(path, serialized)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
    Ok(())
}

#[tauri::command]
pub fn agent_settings_get(
    store: tauri::State<AgentSettingsStore>,
) -> Result<AgentSettings, String> {
    Ok(store.get())
}

#[tauri::command]
pub fn agent_settings_save(
    store: tauri::State<AgentSettingsStore>,
    settings: AgentSettings,
) -> Result<AgentSettings, String> {
    store.save(settings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn load_and_save_round_trip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        fs::write(
            &path,
            r#"
[ai]
host = "server.lan"
port = 11434
model = "qwen3.5:4b"
models = ["qwen3.5:4b", "qwen2.5:7b"]

[agent]
model = "qwen3.5:4b"
runtime = "codex"
"#,
        )
        .unwrap();

        let loaded = load_from_path(&path).unwrap();
        assert_eq!(loaded.host, "server.lan");
        assert_eq!(loaded.model, "qwen3.5:4b");
        assert_eq!(loaded.runtime, "codex");

        save_to_path(&path, &loaded).unwrap();
        let again = load_from_path(&path).unwrap();
        assert_eq!(again.model, "qwen3.5:4b");
        assert_eq!(again.runtime, "codex");
    }

    #[test]
    fn save_creates_missing_parent_and_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nested/config.toml");
        let settings = AgentSettings::default();

        save_to_path(&path, &settings).unwrap();
        assert!(path.is_file());
        let loaded = load_from_path(&path).unwrap();
        assert_eq!(loaded.model, settings.model);
    }

    #[test]
    fn defaults_when_file_missing() {
        let dir = tempdir().unwrap();
        let store = AgentSettingsStore::load(dir.path().join("missing.toml"));
        assert_eq!(store.get().model, AgentSettings::default().model);
    }
}
