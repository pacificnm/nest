//! Runtime-mutable Ollama configuration.

use std::sync::{Arc, RwLock};

use crate::config::OllamaConfig;

/// Thread-safe Ollama settings updated by host applications at runtime.
#[derive(Clone)]
pub struct OllamaSharedConfig {
    inner: Arc<RwLock<OllamaConfig>>,
}

impl OllamaSharedConfig {
    /// Creates a shared config handle from an initial snapshot.
    pub fn new(config: OllamaConfig) -> Self {
        Self {
            inner: Arc::new(RwLock::new(config)),
        }
    }

    /// Returns the current configuration snapshot.
    pub fn snapshot(&self) -> OllamaConfig {
        self.inner
            .read()
            .expect("ollama config lock poisoned")
            .clone()
    }

    /// Replaces the full configuration.
    pub fn set(&self, config: OllamaConfig) {
        *self.inner.write().expect("ollama config lock poisoned") = config;
    }

    /// Updates the inference base URL.
    pub fn set_base_url(&self, base_url: impl Into<String>) {
        self.inner
            .write()
            .expect("ollama config lock poisoned")
            .base_url = trim_trailing_slash(base_url.into());
    }

    /// Updates the default model id.
    pub fn set_model(&self, model: impl Into<String>) {
        self.inner
            .write()
            .expect("ollama config lock poisoned")
            .model = model.into();
    }
}

fn trim_trailing_slash(value: String) -> String {
    value.trim_end_matches('/').to_string()
}
