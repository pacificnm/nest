// src/lib.rs

use std::path::Path;
use nest_error::NestResult;
use toml::Value;

use nest_error::NestError;
/// Reads the Cargo.toml at `app_path` (or its parent directory) and returns the package version.
pub fn app_version(app_path: &Path) -> NestResult<String> {
    // Find the Cargo.toml file – if `app_path` is a directory, look for Cargo.toml inside it.
    let cargo_toml = if app_path.is_dir() {
        app_path.join("Cargo.toml")
    } else {
        // Assume it's a file path; look at its parent.
        app_path.parent().unwrap_or_else(|| Path::new(".")).join("Cargo.toml")
    };
    let content = std::fs::read_to_string(&cargo_toml)
        .map_err(|e| NestError::io(e.to_string()))?;
    let toml: Value = toml::from_str(&content)
        .map_err(|e| NestError::config(e.to_string()))?;
    let version = toml
        .get("package")
        .and_then(|pkg| pkg.get("version"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| NestError::config("missing package.version"))?;
    Ok(version.to_string())
}
