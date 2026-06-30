//! Theme file loading from TOML and JSON.

use std::fs;
use std::path::Path;

use nest_design::ThemeDefinition;

use crate::error::{self, ThemeResult};
use crate::validator::ThemeValidator;

/// Supported on-disk theme file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeFormat {
    /// TOML (`.toml`).
    Toml,
    /// JSON (`.json`).
    Json,
}

/// Loads and validates theme definitions from strings or files.
pub struct ThemeLoader;

impl ThemeLoader {
    /// Parses theme content in the given format and validates it.
    pub fn from_str(content: &str, format: ThemeFormat) -> ThemeResult<ThemeDefinition> {
        let theme = match format {
            ThemeFormat::Toml => toml::from_str(content)
                .map_err(|e| nest_error::NestError::config(format!("invalid TOML: {e}")))?,
            ThemeFormat::Json => serde_json::from_str(content)
                .map_err(|e| nest_error::NestError::config(format!("invalid JSON: {e}")))?,
        };
        ThemeValidator::validate(&theme)?;
        Ok(theme)
    }

    /// Reads a theme file, inferring format from the extension.
    pub fn from_file(path: impl AsRef<Path>) -> ThemeResult<ThemeDefinition> {
        let path = path.as_ref();
        let content = fs::read_to_string(path).map_err(error::io)?;
        let format = infer_format(path)?;
        Self::from_str(&content, format)
    }
}

fn infer_format(path: &Path) -> ThemeResult<ThemeFormat> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    match ext.as_str() {
        "toml" => Ok(ThemeFormat::Toml),
        "json" => Ok(ThemeFormat::Json),
        _ => Err(error::unsupported_format(&path.display().to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nest_design::themes::light;

    fn themes_dir() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../themes")
    }

    #[test]
    fn loads_nest_light_toml() {
        let path = themes_dir().join("nest-light.toml");
        let loaded = ThemeLoader::from_file(&path).unwrap();
        assert_eq!(loaded.id.as_str(), "nest-light");
        assert_eq!(loaded, light());
    }

    #[test]
    fn loads_nest_dark_toml() {
        let path = themes_dir().join("nest-dark.toml");
        let loaded = ThemeLoader::from_file(&path).unwrap();
        assert_eq!(loaded.id.as_str(), "nest-dark");
    }

    #[test]
    fn loads_json_round_trip() {
        let theme = light();
        let json = serde_json::to_string(&theme).unwrap();
        let loaded = ThemeLoader::from_str(&json, ThemeFormat::Json).unwrap();
        assert_eq!(loaded, theme);
    }

    #[test]
    fn unsupported_extension_fails() {
        let path = std::env::temp_dir().join("nest-theme-unsupported.txt");
        std::fs::write(&path, "id = \"x\"\n").unwrap();
        let err = ThemeLoader::from_file(&path).unwrap_err();
        assert_eq!(err.kind(), nest_error::NestErrorKind::Config);
        let _ = std::fs::remove_file(path);
    }
}
