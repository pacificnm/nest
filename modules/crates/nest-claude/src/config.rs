//! Claude client configuration.

use nest_error::{NestError, NestResult};

use crate::codes::NEST_CLAUDE_API_KEY_MISSING;

/// Default Claude API base URL.
pub const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";

/// Default environment variable for the API key.
pub const DEFAULT_API_KEY_ENV: &str = "ANTHROPIC_API_KEY";

/// Default `anthropic-version` header value.
pub const DEFAULT_ANTHROPIC_VERSION: &str = "2023-06-01";

/// Default model when none is specified per-request.
pub const DEFAULT_MODEL: &str = "claude-opus-4-8";

/// Default `max_tokens` when none is specified per-request.
pub const DEFAULT_MAX_TOKENS: u32 = 4096;

/// Resolved Claude client configuration.
#[derive(Debug, Clone)]
pub struct ClaudeConfig {
    /// Anthropic API key.
    pub api_key: String,
    /// API base URL without trailing slash.
    pub base_url: String,
    /// `anthropic-version` header value.
    pub anthropic_version: String,
    /// Model used when a request does not specify one.
    pub default_model: String,
    /// `max_tokens` used when a request does not specify one.
    pub default_max_tokens: u32,
}

impl ClaudeConfig {
    /// Creates configuration from `ANTHROPIC_API_KEY` (or `api_key_env` override).
    pub fn from_env() -> NestResult<Self> {
        Self::builder().build()
    }

    /// Returns a configuration builder.
    pub fn builder() -> ClaudeConfigBuilder {
        ClaudeConfigBuilder::new()
    }
}

/// Programmatic configuration builder.
#[derive(Debug, Clone)]
pub struct ClaudeConfigBuilder {
    api_key: Option<String>,
    api_key_env: String,
    base_url: String,
    anthropic_version: String,
    default_model: String,
    default_max_tokens: u32,
}

impl ClaudeConfigBuilder {
    /// Creates a builder with defaults.
    pub fn new() -> Self {
        Self {
            api_key: None,
            api_key_env: DEFAULT_API_KEY_ENV.to_string(),
            base_url: DEFAULT_BASE_URL.to_string(),
            anthropic_version: DEFAULT_ANTHROPIC_VERSION.to_string(),
            default_model: DEFAULT_MODEL.to_string(),
            default_max_tokens: DEFAULT_MAX_TOKENS,
        }
    }

    /// Sets the API key directly.
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Sets the environment variable used to resolve the API key.
    pub fn api_key_env(mut self, api_key_env: impl Into<String>) -> Self {
        self.api_key_env = api_key_env.into();
        self
    }

    /// Sets the API base URL.
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into().trim_end_matches('/').to_string();
        self
    }

    /// Sets the `anthropic-version` header value.
    pub fn anthropic_version(mut self, version: impl Into<String>) -> Self {
        self.anthropic_version = version.into();
        self
    }

    /// Sets the default model.
    pub fn default_model(mut self, model: impl Into<String>) -> Self {
        self.default_model = model.into();
        self
    }

    /// Sets the default `max_tokens`.
    pub fn default_max_tokens(mut self, max_tokens: u32) -> Self {
        self.default_max_tokens = max_tokens;
        self
    }

    /// Builds the resolved configuration.
    pub fn build(self) -> NestResult<ClaudeConfig> {
        let api_key = resolve_api_key(self.api_key.as_deref(), Some(&self.api_key_env))?;
        Ok(ClaudeConfig {
            api_key,
            base_url: self.base_url,
            anthropic_version: self.anthropic_version,
            default_model: self.default_model,
            default_max_tokens: self.default_max_tokens,
        })
    }
}

impl Default for ClaudeConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Resolves the Claude API key from inline config or environment.
pub fn resolve_api_key(api_key: Option<&str>, api_key_env: Option<&str>) -> NestResult<String> {
    if let Some(api_key) = api_key.map(str::trim).filter(|value| !value.is_empty()) {
        return Ok(api_key.to_string());
    }

    let env_name = api_key_env
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_API_KEY_ENV);

    std::env::var(env_name).map_err(|_| {
        NestError::config(format!("environment variable not set: {env_name}"))
            .with_code(NEST_CLAUDE_API_KEY_MISSING)
            .with_module("nest-claude")
            .with_help(format!("Export {env_name} with your Anthropic API key."))
    })
}

#[cfg(feature = "config")]
mod config_service {
    use nest_config::{ConfigDocument, ConfigService};
    use nest_error::NestResult;
    use serde::Deserialize;

    use super::{
        resolve_api_key, ClaudeConfig, DEFAULT_ANTHROPIC_VERSION, DEFAULT_BASE_URL,
        DEFAULT_MAX_TOKENS, DEFAULT_MODEL,
    };

    #[derive(Debug, Clone, Deserialize)]
    pub(crate) struct ClaudeSection {
        #[serde(default)]
        api_key: Option<String>,
        #[serde(default)]
        api_key_env: Option<String>,
        #[serde(default)]
        base_url: Option<String>,
        #[serde(default)]
        anthropic_version: Option<String>,
        #[serde(default)]
        default_model: Option<String>,
        #[serde(default)]
        default_max_tokens: Option<u32>,
    }

    impl ClaudeConfig {
        /// Loads configuration from a [`ConfigDocument`].
        pub fn from_document(document: &ConfigDocument) -> NestResult<Self> {
            let section: ClaudeSection = document.section("claude")?;
            let api_key =
                resolve_api_key(section.api_key.as_deref(), section.api_key_env.as_deref())?;
            Ok(Self {
                api_key,
                base_url: section
                    .base_url
                    .unwrap_or_else(|| DEFAULT_BASE_URL.to_string())
                    .trim_end_matches('/')
                    .to_string(),
                anthropic_version: section
                    .anthropic_version
                    .unwrap_or_else(|| DEFAULT_ANTHROPIC_VERSION.to_string()),
                default_model: section
                    .default_model
                    .unwrap_or_else(|| DEFAULT_MODEL.to_string()),
                default_max_tokens: section.default_max_tokens.unwrap_or(DEFAULT_MAX_TOKENS),
            })
        }

        /// Loads configuration from a [`ConfigService`].
        pub fn from_config_service(config: &ConfigService) -> NestResult<Self> {
            Self::from_document(config.document())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_uses_explicit_api_key() {
        let config = ClaudeConfig::builder()
            .api_key("test-key")
            .default_model("claude-sonnet-5")
            .build()
            .unwrap();
        assert_eq!(config.api_key, "test-key");
        assert_eq!(config.default_model, "claude-sonnet-5");
        assert_eq!(config.base_url, DEFAULT_BASE_URL);
    }

    #[test]
    fn missing_api_key_errors() {
        // SAFETY: test env var manipulation, single-threaded within this test.
        let env_name = "NEST_CLAUDE_TEST_MISSING_KEY";
        std::env::remove_var(env_name);
        let result = ClaudeConfig::builder().api_key_env(env_name).build();
        assert!(result.is_err());
    }
}
