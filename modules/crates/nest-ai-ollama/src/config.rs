//! Ollama client configuration.

use nest_config::ConfigService;
use nest_error::NestResult;
use serde::Deserialize;

/// Default Ollama HTTP base URL.
pub const DEFAULT_BASE_URL: &str = "http://127.0.0.1:11434";

/// Default model when none is specified on a request.
pub const DEFAULT_MODEL: &str = "smollm2:360m";

/// Resolved Ollama client configuration.
#[derive(Debug, Clone)]
pub struct OllamaConfig {
    /// API base URL without trailing slash.
    pub base_url: String,
    /// Default model id.
    pub model: String,
}

impl OllamaConfig {
    /// Creates configuration with defaults.
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            base_url: trim_trailing_slash(base_url.into()),
            model: model.into(),
        }
    }

    /// Creates configuration from defaults.
    pub fn default_local() -> Self {
        Self::new(DEFAULT_BASE_URL, DEFAULT_MODEL)
    }

    /// Loads `[ai]` or `[ollama]` section from a config service.
    #[cfg(feature = "config")]
    pub fn from_config_service(service: &ConfigService) -> NestResult<Option<Self>> {
        if let Ok(section) = service.section::<AiSection>("ai") {
            if !section.enabled {
                return Ok(None);
            }
            return Ok(Some(section.into_config()));
        }

        if let Ok(section) = service.section::<OllamaSection>("ollama") {
            if section.enabled == Some(false) {
                return Ok(None);
            }
            return Ok(Some(section.into_config()));
        }

        Ok(None)
    }
}

/// `[ai]` section when `provider = "ollama"`.
#[cfg(feature = "config")]
#[derive(Debug, Clone, Deserialize)]
pub struct AiSection {
    /// Whether AI filename guessing is enabled.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Provider id (`ollama` only in v0.1).
    #[serde(default = "default_provider")]
    pub provider: String,
    /// Inference base URL.
    #[serde(default = "default_base_url")]
    pub base_url: String,
    /// Default model id.
    #[serde(default = "default_model")]
    pub model: String,
}

#[cfg(feature = "config")]
impl AiSection {
    fn into_config(self) -> OllamaConfig {
        if self.provider != "ollama" {
            return OllamaConfig::new(self.base_url, self.model);
        }
        OllamaConfig::new(self.base_url, self.model)
    }
}

/// Legacy `[ollama]` section.
#[cfg(feature = "config")]
#[derive(Debug, Clone, Deserialize)]
pub struct OllamaSection {
    /// Whether Ollama is enabled.
    pub enabled: Option<bool>,
    /// Inference base URL.
    #[serde(default = "default_base_url")]
    pub base_url: String,
    /// Default model id.
    #[serde(default = "default_model")]
    pub model: String,
}

#[cfg(feature = "config")]
impl OllamaSection {
    fn into_config(self) -> OllamaConfig {
        OllamaConfig::new(self.base_url, self.model)
    }
}

fn default_enabled() -> bool {
    true
}

fn default_provider() -> String {
    "ollama".into()
}

fn default_base_url() -> String {
    DEFAULT_BASE_URL.into()
}

fn default_model() -> String {
    DEFAULT_MODEL.into()
}

fn trim_trailing_slash(value: String) -> String {
    value.trim_end_matches('/').to_string()
}
