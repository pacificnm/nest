//! Airtable configuration loaded from TOML or programmatic builders.

use std::collections::HashMap;

use nest_config::{ConfigDocument, ConfigService};
use nest_error::{NestError, NestResult};
use serde::Deserialize;

use crate::codes::NEST_AIRTABLE_TOKEN_MISSING;

/// Default Airtable REST API base URL.
pub const DEFAULT_API_URL: &str = "https://api.airtable.com/v0";

/// Default environment variable for the API token.
pub const DEFAULT_TOKEN_ENV: &str = "AIRTABLE_TOKEN";

/// Per-table configuration.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AirtableTableConfig {
    /// Airtable table id (`tbl…`).
    pub table_id: String,
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
    token_env: Option<String>,
    base_id: String,
    #[serde(default)]
    tables: HashMap<String, AirtableTableConfig>,
}

/// Resolved Airtable client configuration.
#[derive(Debug, Clone)]
pub struct AirtableConfig {
    /// API base URL (without trailing slash).
    pub api_url: String,
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

    /// Creates configuration from a parsed section and environment token.
    pub(crate) fn from_section(section: AirtableSection) -> NestResult<Self> {
        let token_env = section
            .token_env
            .unwrap_or_else(|| DEFAULT_TOKEN_ENV.to_string());
        let token = std::env::var(&token_env).map_err(|_| {
            NestError::config(format!("environment variable not set: {token_env}"))
                .with_code(NEST_AIRTABLE_TOKEN_MISSING)
                .with_module("nest-airtable")
                .with_help(format!("Export {token_env} with your Airtable personal access token."))
        })?;

        Ok(Self {
            api_url: section
                .api_url
                .unwrap_or_else(|| DEFAULT_API_URL.to_string())
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
                primary_key_field,
            },
        );
        self
    }

    /// Builds the resolved configuration.
    pub fn build(self) -> NestResult<AirtableConfig> {
        Ok(AirtableConfig {
            api_url: self.api_url,
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
