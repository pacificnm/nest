//! Nest Desktop's own settings file location (distinct from `nest_root.rs`,
//! which locates the Nest *repository* for docs/apps/files browsing).

use std::path::PathBuf;

/// Environment override for the active `config.toml`.
const CONFIG_ENV: &str = "NEST_DESKTOP_CONFIG";

/// Returns the Nest Desktop XDG config directory (`~/.config/nest-desktop` on Linux).
pub fn nest_desktop_config_dir() -> PathBuf {
    dirs::config_dir()
        .map(|dir| dir.join("nest-desktop"))
        .or_else(|| {
            std::env::var_os("HOME")
                .map(|home| PathBuf::from(home).join(".config").join("nest-desktop"))
        })
        .unwrap_or_else(|| PathBuf::from(".config").join("nest-desktop"))
}

/// Locates Nest Desktop's `config.toml`, honoring `NEST_DESKTOP_CONFIG` for tests
/// and overrides. Does not require the file to exist yet.
pub fn resolve_config_path() -> PathBuf {
    if let Ok(raw) = std::env::var(CONFIG_ENV) {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    nest_desktop_config_dir().join("config.toml")
}
