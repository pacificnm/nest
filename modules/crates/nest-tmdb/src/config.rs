//! TMDB client configuration.

use nest_error::{NestError, NestResult};

use crate::codes::NEST_TMDB_API_KEY_MISSING;

/// Default TMDB v3 API base URL.
pub const DEFAULT_BASE_URL: &str = "https://api.themoviedb.org/3";

/// Default TMDB image CDN base URL (used until `/configuration` is loaded).
pub const DEFAULT_IMAGE_BASE_URL: &str = "https://image.tmdb.org/t/p/";

/// Default environment variable for the API key.
pub const DEFAULT_API_KEY_ENV: &str = "TMDB_API_KEY";

/// Default response language.
pub const DEFAULT_LANGUAGE: &str = "en-US";

/// Resolved TMDB client configuration.
#[derive(Debug, Clone)]
pub struct TmdbConfig {
    /// TMDB v3 API key.
    pub api_key: String,
    /// API base URL without trailing slash.
    pub base_url: String,
    /// Image CDN base URL without trailing slash.
    pub image_base_url: String,
    /// Response language (TMDB `language` query param).
    pub language: String,
    /// Optional region filter.
    pub region: Option<String>,
}

impl TmdbConfig {
    /// Creates configuration from `TMDB_API_KEY` (or `api_key_env` override).
    pub fn from_env() -> NestResult<Self> {
        Self::builder().build()
    }

    /// Returns a configuration builder.
    pub fn builder() -> TmdbConfigBuilder {
        TmdbConfigBuilder::new()
    }
}

/// Programmatic configuration builder.
#[derive(Debug, Clone)]
pub struct TmdbConfigBuilder {
    api_key: Option<String>,
    api_key_env: String,
    base_url: String,
    image_base_url: String,
    language: String,
    region: Option<String>,
}

impl TmdbConfigBuilder {
    /// Creates a builder with defaults.
    pub fn new() -> Self {
        Self {
            api_key: None,
            api_key_env: DEFAULT_API_KEY_ENV.to_string(),
            base_url: DEFAULT_BASE_URL.to_string(),
            image_base_url: DEFAULT_IMAGE_BASE_URL.to_string(),
            language: DEFAULT_LANGUAGE.to_string(),
            region: None,
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

    /// Sets the image CDN base URL.
    pub fn image_base_url(mut self, image_base_url: impl Into<String>) -> Self {
        self.image_base_url = image_base_url.into().trim_end_matches('/').to_string();
        self.image_base_url.push('/');
        self
    }

    /// Sets the response language.
    pub fn language(mut self, language: impl Into<String>) -> Self {
        self.language = language.into();
        self
    }

    /// Sets an optional region.
    pub fn region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    /// Builds the resolved configuration.
    pub fn build(self) -> NestResult<TmdbConfig> {
        let api_key = resolve_api_key(self.api_key.as_deref(), Some(&self.api_key_env))?;
        Ok(TmdbConfig {
            api_key,
            base_url: self.base_url,
            image_base_url: self.image_base_url,
            language: self.language,
            region: self.region,
        })
    }
}

impl Default for TmdbConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Resolves the TMDB API key from inline config or environment.
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
            .with_code(NEST_TMDB_API_KEY_MISSING)
            .with_module("nest-tmdb")
            .with_help(format!("Export {env_name} with your TMDB API key."))
    })
}

#[cfg(feature = "config")]
mod config_service {
    use nest_config::{ConfigDocument, ConfigService};
    use nest_error::NestResult;
    use serde::Deserialize;

    use super::{
        resolve_api_key, DEFAULT_BASE_URL, DEFAULT_IMAGE_BASE_URL,
        DEFAULT_LANGUAGE, TmdbConfig,
    };

    #[derive(Debug, Clone, Deserialize)]
    pub(crate) struct TmdbSection {
        #[serde(default)]
        api_key: Option<String>,
        #[serde(default)]
        api_key_env: Option<String>,
        #[serde(default)]
        base_url: Option<String>,
        #[serde(default)]
        image_base_url: Option<String>,
        #[serde(default)]
        language: Option<String>,
        #[serde(default)]
        region: Option<String>,
    }

    impl TmdbConfig {
        /// Loads configuration from a [`ConfigDocument`].
        pub fn from_document(document: &ConfigDocument) -> NestResult<Self> {
            let section: TmdbSection = document.section("tmdb")?;
            let api_key = resolve_api_key(
                section.api_key.as_deref(),
                section.api_key_env.as_deref(),
            )?;
            Ok(Self {
                api_key,
                base_url: section
                    .base_url
                    .unwrap_or_else(|| DEFAULT_BASE_URL.to_string())
                    .trim_end_matches('/')
                    .to_string(),
                image_base_url: section
                    .image_base_url
                    .unwrap_or_else(|| DEFAULT_IMAGE_BASE_URL.to_string()),
                language: section
                    .language
                    .unwrap_or_else(|| DEFAULT_LANGUAGE.to_string()),
                region: section.region,
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
        let config = TmdbConfig::builder()
            .api_key("test-key")
            .language("fr-FR")
            .build()
            .unwrap();
        assert_eq!(config.api_key, "test-key");
        assert_eq!(config.language, "fr-FR");
    }
}
