//! Claude API key — load / save from Nest Desktop's `config.toml`.
//!
//! Used by the Claude Config app to persist the Anthropic API key so the
//! Skills/Agents viewer can authenticate. Mirrors `agent_settings.rs`'s
//! load/save-from-TOML pattern.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use toml::Value;

/// Claude API settings persisted in the `[claude]` config section.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeSettings {
    /// The Anthropic API key (`sk-ant-...`), stored in plaintext in
    /// `config.toml` — same trust model as the existing Airtable token setup.
    pub api_key: String,
}

/// Managed Tauri state for Claude API settings.
pub struct ClaudeSettingsStore {
    path: PathBuf,
    settings: Mutex<ClaudeSettings>,
}

impl ClaudeSettingsStore {
    /// Loads settings from `path`, or defaults (empty key) when missing.
    pub fn load(path: PathBuf) -> Self {
        let settings = load_from_path(&path).unwrap_or_default();
        Self {
            path,
            settings: Mutex::new(settings),
        }
    }

    /// Returns a clone of the current settings.
    pub fn get(&self) -> ClaudeSettings {
        self.settings.lock().expect("claude settings mutex").clone()
    }

    /// Persists `settings` to disk and updates in-memory state.
    pub fn save(&self, settings: ClaudeSettings) -> Result<ClaudeSettings, String> {
        save_to_path(&self.path, &settings)?;
        *self.settings.lock().expect("claude settings mutex") = settings.clone();
        Ok(settings)
    }
}

fn load_from_path(path: &Path) -> Option<ClaudeSettings> {
    let text = fs::read_to_string(path).ok()?;
    let root: Value = text.parse().ok()?;
    let mut settings = ClaudeSettings::default();

    if let Some(claude) = root.get("claude").and_then(Value::as_table) {
        if let Some(api_key) = claude.get("api_key").and_then(Value::as_str) {
            settings.api_key = api_key.trim().to_string();
        }
    }

    Some(settings)
}

fn save_to_path(path: &Path, settings: &ClaudeSettings) -> Result<(), String> {
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

    let claude = table
        .entry("claude")
        .or_insert_with(|| Value::Table(toml::map::Map::new()));
    let claude_table = claude
        .as_table_mut()
        .ok_or_else(|| "[claude] must be a table".to_string())?;
    claude_table.insert(
        "api_key".into(),
        Value::String(settings.api_key.trim().to_string()),
    );

    let serialized = toml::to_string_pretty(&root)
        .map_err(|error| format!("failed to serialize config: {error}"))?;
    fs::write(path, serialized)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))?;
    Ok(())
}

#[tauri::command]
pub fn claude_settings_get(
    store: tauri::State<ClaudeSettingsStore>,
) -> Result<ClaudeSettings, String> {
    Ok(store.get())
}

#[tauri::command]
pub fn claude_settings_save(
    store: tauri::State<ClaudeSettingsStore>,
    settings: ClaudeSettings,
) -> Result<ClaudeSettings, String> {
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
[claude]
api_key = "sk-ant-test-123"
"#,
        )
        .unwrap();

        let loaded = load_from_path(&path).unwrap();
        assert_eq!(loaded.api_key, "sk-ant-test-123");

        save_to_path(&path, &loaded).unwrap();
        let again = load_from_path(&path).unwrap();
        assert_eq!(again.api_key, "sk-ant-test-123");
    }

    #[test]
    fn defaults_when_file_missing() {
        let dir = tempdir().unwrap();
        let store = ClaudeSettingsStore::load(dir.path().join("missing.toml"));
        assert_eq!(store.get().api_key, "");
    }

    #[test]
    fn save_preserves_other_top_level_sections() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.toml");
        fs::write(&path, "[agent]\nmodel = \"qwen3.5:2b\"\n").unwrap();

        save_to_path(
            &path,
            &ClaudeSettings {
                api_key: "sk-ant-abc".to_string(),
            },
        )
        .unwrap();

        let content = fs::read_to_string(&path).unwrap();
        assert!(content.contains("qwen3.5:2b"));
        assert!(content.contains("sk-ant-abc"));
    }
}
