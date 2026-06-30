//! Configuration file format detection and parsing.

use std::path::Path;

use nest_error::{NestError, NestResult};

use crate::codes::NEST_CONFIG_UNSUPPORTED_FORMAT;
use crate::document::ConfigDocument;

/// Supported configuration file formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConfigFormat {
    /// TOML format.
    Toml,
    /// JSON format.
    #[cfg(feature = "json")]
    Json,
    /// Infer from file extension; TOML for in-memory sources.
    #[default]
    Auto,
}

impl ConfigFormat {
    /// Resolves the format for a file path when using [`ConfigFormat::Auto`].
    pub fn resolve_for_path(path: &Path) -> NestResult<Self> {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("toml") => Ok(Self::Toml),
            #[cfg(feature = "json")]
            Some("json") => Ok(Self::Json),
            Some(ext) => Err(
                NestError::config(format!("unsupported configuration format: .{ext}"))
                    .with_code(NEST_CONFIG_UNSUPPORTED_FORMAT)
                    .with_operation(format!("path: {}", path.display())),
            ),
            None => Err(
                NestError::config("configuration file has no extension")
                    .with_code(NEST_CONFIG_UNSUPPORTED_FORMAT)
                    .with_operation(format!("path: {}", path.display())),
            ),
        }
    }

    /// Parses file content into a document.
    pub fn parse(self, content: &str, path: Option<&Path>) -> NestResult<ConfigDocument> {
        let format = match self {
            Self::Auto => {
                let path = path.ok_or_else(|| {
                    NestError::config("cannot auto-detect format without a file path")
                        .with_code(NEST_CONFIG_UNSUPPORTED_FORMAT)
                })?;
                Self::resolve_for_path(path)?
            }
            other => other,
        };

        match format {
            Self::Toml => ConfigDocument::parse_toml(content),
            #[cfg(feature = "json")]
            Self::Json => ConfigDocument::parse_json(content),
            Self::Auto => unreachable!("auto format is resolved before parsing"),
        }
    }
}
