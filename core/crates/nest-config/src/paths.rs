//! Default configuration file search paths.

use std::path::PathBuf;

/// Returns default configuration search paths for an application.
pub fn default_search_paths(app_name: &str) -> Vec<PathBuf> {
    let mut paths = vec![PathBuf::from("config.toml"), PathBuf::from("config/config.toml")];
    if let Some(home) = home_config_path(app_name) {
        paths.push(home);
    }
    paths
}

/// Returns the first existing path from default search locations.
pub fn resolve_search(app_name: &str) -> Option<PathBuf> {
    default_search_paths(app_name)
        .into_iter()
        .find(|path| path.exists())
}

fn home_config_path(app_name: &str) -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| {
        PathBuf::from(home)
            .join(".config")
            .join(app_name)
            .join("config.toml")
    })
}
