//! Ollama client configuration.

#![allow(clippy::result_large_err)]

use nest_config::ConfigService;
use nest_error::NestResult;
use serde::Deserialize;

/// Default Ollama HTTP base URL.
pub const DEFAULT_BASE_URL: &str = "http://127.0.0.1:11434";

/// Default model when none is specified on a request.
pub const DEFAULT_MODEL: &str = "smollm2:360m";

/// Default Ollama HTTP port.
pub const DEFAULT_PORT: u16 = 11434;

/// Resolved Ollama client configuration.
#[derive(Debug, Clone)]
pub struct OllamaConfig {
    /// API base URL without trailing slash.
    pub base_url: String,
    /// Default model id.
    pub model: String,
    /// Context window size in tokens (Ollama `options.num_ctx`). `None` uses
    /// the model's/Ollama's own default, which is often far smaller than a
    /// model actually supports.
    pub num_ctx: Option<u32>,
    /// Sampling temperature (Ollama `options.temperature`). `None` uses
    /// Ollama's default.
    pub temperature: Option<f32>,
    /// Enables extended thinking/reasoning mode (Ollama `think`) on models
    /// that support it (e.g. qwen3, deepseek-r1). Improves tool-selection
    /// accuracy at the cost of latency — off by default.
    pub think: bool,
}

impl OllamaConfig {
    /// Creates configuration with defaults.
    pub fn new(base_url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            base_url: trim_trailing_slash(base_url.into()),
            model: model.into(),
            num_ctx: None,
            temperature: None,
            think: false,
        }
    }

    /// Sets the context window size in tokens.
    pub fn with_num_ctx(mut self, num_ctx: u32) -> Self {
        self.num_ctx = Some(num_ctx);
        self
    }

    /// Sets the sampling temperature.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Enables (or disables) extended thinking/reasoning mode.
    pub fn with_think(mut self, think: bool) -> Self {
        self.think = think;
        self
    }

    /// Creates configuration from defaults.
    pub fn default_local() -> Self {
        Self::new(DEFAULT_BASE_URL, DEFAULT_MODEL)
    }

    /// Builds a base URL from host and port.
    pub fn base_url_from_host(host: &str, port: u16) -> String {
        format!(
            "http://{}:{}",
            host.trim()
                .trim_start_matches("http://")
                .trim_start_matches("https://"),
            port
        )
    }

    /// Parses host and port from a base URL.
    pub fn host_port_from_base_url(base_url: &str) -> (String, u16) {
        let trimmed = base_url.trim().trim_end_matches('/');
        let without_scheme = trimmed
            .strip_prefix("http://")
            .or_else(|| trimmed.strip_prefix("https://"))
            .unwrap_or(trimmed);

        match without_scheme.split_once(':') {
            Some((host, port)) => (host.to_string(), port.parse().unwrap_or(DEFAULT_PORT)),
            None => (without_scheme.to_string(), DEFAULT_PORT),
        }
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
    /// Agent host (IP or hostname). When set, overrides `base_url` host/port.
    pub host: Option<String>,
    /// Agent HTTP port.
    #[serde(default = "default_port")]
    pub port: u16,
    /// Available model ids for the agent.
    #[serde(default)]
    pub models: Vec<String>,
    /// Context window size in tokens (Ollama `options.num_ctx`).
    pub num_ctx: Option<u32>,
    /// Sampling temperature (Ollama `options.temperature`).
    pub temperature: Option<f32>,
    /// Enables extended thinking/reasoning mode (Ollama `think`).
    #[serde(default)]
    pub think: bool,
}

#[cfg(feature = "config")]
impl AiSection {
    /// Resolves the effective Ollama HTTP base URL.
    pub fn resolved_base_url(&self) -> String {
        if let Some(host) = &self.host {
            if !host.trim().is_empty() {
                return OllamaConfig::base_url_from_host(host, self.port);
            }
        }
        trim_trailing_slash(self.base_url.clone())
    }

    fn into_config(self) -> OllamaConfig {
        let mut config = OllamaConfig::new(self.resolved_base_url(), self.model);
        config.num_ctx = self.num_ctx;
        config.temperature = self.temperature;
        config.think = self.think;
        config
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
    /// Context window size in tokens (Ollama `options.num_ctx`).
    pub num_ctx: Option<u32>,
    /// Sampling temperature (Ollama `options.temperature`).
    pub temperature: Option<f32>,
    /// Enables extended thinking/reasoning mode (Ollama `think`).
    #[serde(default)]
    pub think: bool,
}

#[cfg(feature = "config")]
impl OllamaSection {
    fn into_config(self) -> OllamaConfig {
        let mut config = OllamaConfig::new(self.base_url, self.model);
        config.num_ctx = self.num_ctx;
        config.temperature = self.temperature;
        config.think = self.think;
        config
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

fn default_port() -> u16 {
    DEFAULT_PORT
}

fn trim_trailing_slash(value: String) -> String {
    value.trim_end_matches('/').to_string()
}

#[cfg(all(test, feature = "config"))]
mod tests {
    use super::*;
    use nest_config::{ConfigDocument, ConfigService, LoadedConfig};

    fn service_from_toml(toml: &str) -> ConfigService {
        let document = ConfigDocument::parse_toml(toml).unwrap();
        let loaded = LoadedConfig {
            document: document.clone(),
            source: nest_config::ConfigSource::Memory(document),
            path: None,
        };
        ConfigService::new(loaded)
    }

    #[test]
    fn from_config_service_reads_num_ctx_temperature_and_think() {
        let service = service_from_toml(
            r#"
[ai]
enabled = true
provider = "ollama"
model = "qwen3:32b-q4_K_M"
num_ctx = 40960
temperature = 0.2
think = true
"#,
        );

        let config = OllamaConfig::from_config_service(&service).unwrap().unwrap();

        assert_eq!(config.model, "qwen3:32b-q4_K_M");
        assert_eq!(config.num_ctx, Some(40960));
        assert_eq!(config.temperature, Some(0.2));
        assert!(config.think);
    }

    #[test]
    fn from_config_service_defaults_num_ctx_temperature_and_think_when_absent() {
        let service = service_from_toml(
            r#"
[ai]
enabled = true
provider = "ollama"
model = "llama3"
"#,
        );

        let config = OllamaConfig::from_config_service(&service).unwrap().unwrap();

        assert_eq!(config.num_ctx, None);
        assert_eq!(config.temperature, None);
        assert!(!config.think);
    }
}
