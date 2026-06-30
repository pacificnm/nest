//! Configuration loading.

use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

use nest_error::{NestError, NestResult};
use tracing::{debug, warn};

use crate::codes::{NEST_CONFIG_NOT_FOUND, NEST_CONFIG_READ_FAILED};
use crate::document::ConfigDocument;
use crate::format::ConfigFormat;
use crate::paths::resolve_search;
use crate::source::ConfigSource;

/// Result of a successful configuration load.
#[derive(Debug, Clone)]
pub struct LoadedConfig {
    /// Parsed configuration document.
    pub document: ConfigDocument,
    /// Source that produced the document.
    pub source: ConfigSource,
    /// Resolved file path when loaded from disk.
    pub path: Option<PathBuf>,
}

/// Loads configuration from files or in-memory sources.
#[derive(Debug, Clone)]
pub struct ConfigLoader {
    app_name: String,
    source: ConfigSource,
    format: ConfigFormat,
}

impl ConfigLoader {
    /// Creates a loader for the given application name.
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
            source: ConfigSource::SearchDefaults,
            format: ConfigFormat::Auto,
        }
    }

    /// Sets the configuration source.
    pub fn source(mut self, source: ConfigSource) -> Self {
        self.source = source;
        self
    }

    /// Sets the configuration format.
    pub fn format(mut self, format: ConfigFormat) -> Self {
        self.format = format;
        self
    }

    /// Creates a loader for an explicit file path or default search.
    pub fn file_or_search(app_name: &str, explicit: Option<PathBuf>) -> Self {
        let source = match explicit {
            Some(path) => ConfigSource::File(path),
            None => ConfigSource::SearchDefaults,
        };
        Self::new(app_name).source(source)
    }

    /// Loads configuration according to the configured source and format.
    pub fn load(self) -> NestResult<LoadedConfig> {
        debug!(
            config.app_name = %self.app_name,
            config.source = self.source.label(),
            "config load"
        );

        match self.source {
            ConfigSource::File(path) => load_file(path, self.format),
            ConfigSource::SearchDefaults => load_search_defaults(&self.app_name, self.format),
            ConfigSource::Memory(document) => Ok(LoadedConfig {
                document: document.clone(),
                source: ConfigSource::Memory(document),
                path: None,
            }),
        }
    }
}

fn load_search_defaults(app_name: &str, format: ConfigFormat) -> NestResult<LoadedConfig> {
    if let Some(path) = resolve_search(app_name) {
        let mut loaded = load_file(path, format)?;
        loaded.source = ConfigSource::SearchDefaults;
        return Ok(loaded);
    }

    warn!(config.app_name = %app_name, "no configuration file found in default search paths");
    Ok(LoadedConfig {
        document: ConfigDocument::empty(),
        source: ConfigSource::SearchDefaults,
        path: None,
    })
}

fn load_file(path: PathBuf, format: ConfigFormat) -> NestResult<LoadedConfig> {
    if !path.exists() {
        return Err(
            NestError::config(format!("configuration file not found: {}", path.display()))
                .with_code(NEST_CONFIG_NOT_FOUND)
                .with_operation(format!("path: {}", path.display()))
                .with_help("Check the path and try again."),
        );
    }

    let content = fs::read_to_string(&path).map_err(|error| {
        let code = if error.kind() == ErrorKind::NotFound {
            NEST_CONFIG_NOT_FOUND
        } else {
            NEST_CONFIG_READ_FAILED
        };
        NestError::config(format!("failed to read configuration file: {}", path.display()))
            .with_code(code)
            .with_operation(format!("path: {}", path.display()))
            .with_source(error)
    })?;

    let document = format.parse(&content, Some(&path))?;
    debug!(
        config.path = %path.display(),
        config.format = ?format,
        "config loaded"
    );

    Ok(LoadedConfig {
        document,
        source: ConfigSource::File(path.clone()),
        path: Some(path),
    })
}
