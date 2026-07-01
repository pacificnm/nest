//! Airtable configuration loaded from TOML or programmatic builders.

use std::collections::HashMap;

use nest_config::{ConfigDocument, ConfigService};
use nest_error::{NestError, NestResult};
use serde::Deserialize;

use crate::codes::NEST_AIRTABLE_TOKEN_MISSING;

/// Default Airtable Meta API base URL.
pub const DEFAULT_META_API_URL: &str = "https://api.airtable.com/v0/meta";

/// Default Airtable REST API base URL.
pub const DEFAULT_API_URL: &str = "https://api.airtable.com/v0";

/// Default environment variable for the API token.
pub const DEFAULT_TOKEN_ENV: &str = "AIRTABLE_TOKEN";

/// Per-table configuration.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AirtableTableConfig {
    /// Airtable table id (`tbl…`).
    pub table_id: String,
    /// When true, included in bulk sync operations (product config; ignored by the client).
    #[serde(default)]
    pub sync: bool,
    /// Optional primary key field name for sync tooling.
    #[serde(default)]
    pub primary_key_field: Option<String>,
}

/// Root `[airtable]` configuration section.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct AirtableSection {
    #[serde(default)]
    api_url: Option<String>,
    #[serde(default)]
    meta_api_url: Option<String>,
    #[serde(default)]
    token: Option<String>,
    #[serde(default)]
    token_env: Option<String>,
    base_id: String,
    #[serde(default)]
    tables: HashMap<String, AirtableTableConfig>,
}

/// Returns true when a value looks like a literal secret rather than an environment variable name.
pub fn looks_like_secret(value: &str) -> bool {
    let value = value.trim();
    if value.is_empty() {
        return false;
    }

    value.starts_with("pat")
        || value.starts_with("key")
        || value.starts_with("crsr_")
        || value.len() > 20
}

/// Resolves the Airtable bearer token from inline config or environment.
///
/// Resolution order: non-empty `token`, literal `token_env` value (secret heuristic),
/// then `std::env::var` using `token_env` or [`DEFAULT_TOKEN_ENV`].
pub fn resolve_airtable_token(
    token: Option<&str>,
    token_env: Option<&str>,
) -> NestResult<String> {
    if let Some(token) = token.map(str::trim).filter(|value| !value.is_empty()) {
        return Ok(token.to_string());
    }

    let token_env = token_env
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(DEFAULT_TOKEN_ENV);

    if looks_like_secret(token_env) {
        return Ok(token_env.to_string());
    }

    std::env::var(token_env).map_err(|_| {
        NestError::config(format!("environment variable not set: {token_env}"))
            .with_code(NEST_AIRTABLE_TOKEN_MISSING)
            .with_module("nest-airtable")
            .with_help(format!("Export {token_env} with your Airtable personal access token."))
    })
}

/// Resolved Airtable client configuration.
#[derive(Debug, Clone)]
pub struct AirtableConfig {
    /// API base URL (without trailing slash).
    pub api_url: String,
    /// Meta API base URL (without trailing slash).
    pub meta_api_url: String,
    /// Environment variable name holding the bearer token.
    pub token_env: String,
    /// Airtable base id (`app…`).
    pub base_id: String,
    /// Resolved bearer token.
    pub token: String,
    /// Logical table name → table configuration.
    pub tables: HashMap<String, AirtableTableConfig>,
}

impl AirtableConfig {
    /// Loads configuration from a [`ConfigDocument`].
    pub fn from_document(document: &ConfigDocument) -> NestResult<Self> {
        let section: AirtableSection = document.section("airtable")?;
        Self::from_section(section)
    }

    /// Loads configuration from a [`ConfigService`].
    pub fn from_config_service(config: &ConfigService) -> NestResult<Self> {
        Self::from_document(config.document())
    }

    /// Creates configuration from a parsed section and resolved token.
    pub(crate) fn from_section(section: AirtableSection) -> NestResult<Self> {
        let token = resolve_airtable_token(section.token.as_deref(), section.token_env.as_deref())?;
        let token_env = section
            .token_env
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_TOKEN_ENV.to_string());

        Ok(Self {
            api_url: section
                .api_url
                .unwrap_or_else(|| DEFAULT_API_URL.to_string())
                .trim_end_matches('/')
                .to_string(),
            meta_api_url: section
                .meta_api_url
                .unwrap_or_else(|| DEFAULT_META_API_URL.to_string())
                .trim_end_matches('/')
                .to_string(),
            token_env,
            base_id: section.base_id,
            token,
            tables: section.tables,
        })
    }

    /// Returns table configuration for a logical table name.
    pub fn table(&self, name: &str) -> NestResult<&AirtableTableConfig> {
        self.tables
            .get(name)
            .ok_or_else(|| crate::error::table_not_found(name))
    }

    /// Builder for tests and programmatic setup.
    pub fn builder(base_id: impl Into<String>, token: impl Into<String>) -> AirtableConfigBuilder {
        AirtableConfigBuilder::new(base_id, token)
    }
}

/// Programmatic configuration builder.
#[derive(Debug, Clone)]
pub struct AirtableConfigBuilder {
    api_url: String,
    meta_api_url: String,
    token_env: String,
    base_id: String,
    token: String,
    tables: HashMap<String, AirtableTableConfig>,
}

impl AirtableConfigBuilder {
    /// Creates a builder with base id and token.
    pub fn new(base_id: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            api_url: DEFAULT_API_URL.to_string(),
            meta_api_url: DEFAULT_META_API_URL.to_string(),
            token_env: DEFAULT_TOKEN_ENV.to_string(),
            base_id: base_id.into(),
            token: token.into(),
            tables: HashMap::new(),
        }
    }

    /// Sets the API base URL.
    pub fn api_url(mut self, api_url: impl Into<String>) -> Self {
        self.api_url = api_url.into().trim_end_matches('/').to_string();
        self
    }

    /// Sets the Meta API base URL.
    pub fn meta_api_url(mut self, meta_api_url: impl Into<String>) -> Self {
        self.meta_api_url = meta_api_url.into().trim_end_matches('/').to_string();
        self
    }

    /// Sets the token environment variable name (metadata only when token is set directly).
    pub fn token_env(mut self, token_env: impl Into<String>) -> Self {
        self.token_env = token_env.into();
        self
    }

    /// Registers a logical table.
    pub fn table(
        mut self,
        name: impl Into<String>,
        table_id: impl Into<String>,
        primary_key_field: Option<String>,
    ) -> Self {
        self.tables.insert(
            name.into(),
            AirtableTableConfig {
                table_id: table_id.into(),
                sync: false,
                primary_key_field,
            },
        );
        self
    }

    /// Builds the resolved configuration.
    pub fn build(self) -> NestResult<AirtableConfig> {
        Ok(AirtableConfig {
            api_url: self.api_url,
            meta_api_url: self.meta_api_url,
            token_env: self.token_env,
            base_id: self.base_id,
            token: self.token,
            tables: self.tables,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_inline_token_without_env_var() {
        let document = ConfigDocument::parse_toml(
            r#"
[airtable]
token = "pat-inline"
base_id = "appTEST"

[airtable.tables.assets]
table_id = "tblASSETS"
"#,
        )
        .unwrap();

        let config = AirtableConfig::from_document(&document).unwrap();
        assert_eq!(config.token, "pat-inline");
    }

    #[test]
    fn resolves_literal_token_env_value() {
        let document = ConfigDocument::parse_toml(
            r#"
[airtable]
token_env = "pat-from-config"
base_id = "appTEST"

[airtable.tables.assets]
table_id = "tblASSETS"
"#,
        )
        .unwrap();

        let config = AirtableConfig::from_document(&document).unwrap();
        assert_eq!(config.token, "pat-from-config");
    }

    #[test]
    fn parses_sync_field_on_tables() {
        let document = ConfigDocument::parse_toml(
            r#"
[airtable]
token = "pat-test"
base_id = "appTEST"

[airtable.tables.assets]
table_id = "tblASSETS"
sync = true
"#,
        )
        .unwrap();

        let config = AirtableConfig::from_document(&document).unwrap();
        assert!(config.table("assets").unwrap().sync);
    }

    #[test]
    fn parses_nested_table_sections() {
        let document = ConfigDocument::parse_toml(
            r#"
[airtable]
api_url = "https://api.airtable.com/v0"
token_env = "AIRTABLE_TOKEN"
base_id = "appTEST"

[airtable.tables.assets]
table_id = "tblASSETS"
primary_key_field = "Asset ID"
"#,
        )
        .unwrap();

        std::env::set_var("AIRTABLE_TOKEN", "pat-test");
        let config = AirtableConfig::from_document(&document).unwrap();
        assert_eq!(config.base_id, "appTEST");
        let assets = config.table("assets").unwrap();
        assert_eq!(assets.table_id, "tblASSETS");
        assert_eq!(
            assets.primary_key_field.as_deref(),
            Some("Asset ID")
        );
    }
}
