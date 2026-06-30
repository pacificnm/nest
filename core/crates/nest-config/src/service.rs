//! Loaded configuration service.

use std::path::{Path, PathBuf};

use nest_error::NestResult;
use serde::de::DeserializeOwned;

use crate::document::ConfigDocument;
use crate::loader::LoadedConfig;
use crate::source::ConfigSource;

/// Application configuration exposed to modules and commands.
#[derive(Clone)]
pub struct ConfigService {
    document: ConfigDocument,
    source: ConfigSource,
    path: Option<PathBuf>,
}

impl ConfigService {
    /// Creates a service from a loaded configuration result.
    pub fn new(loaded: LoadedConfig) -> Self {
        Self {
            document: loaded.document,
            source: loaded.source,
            path: loaded.path,
        }
    }

    /// Returns the underlying document.
    pub fn document(&self) -> &ConfigDocument {
        &self.document
    }

    /// Returns the resolved configuration file path, if any.
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    /// Returns the configuration source kind.
    pub fn source(&self) -> &ConfigSource {
        &self.source
    }

    /// Returns whether a section exists.
    pub fn has_section(&self, name: &str) -> bool {
        self.document.has_section(name)
    }

    /// Deserializes a required configuration section.
    pub fn section<T: DeserializeOwned>(&self, name: &str) -> NestResult<T> {
        self.document.section(name)
    }

    /// Deserializes an optional configuration section.
    pub fn optional_section<T: DeserializeOwned>(&self, name: &str) -> NestResult<Option<T>> {
        self.document.optional_section(name)
    }
}
