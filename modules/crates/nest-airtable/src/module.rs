//! Registers [`crate::AirtableClient`] via Nest module configuration.

use nest_config::ConfigService;
use nest_core::{AppBuilder, Module, ModuleId, NestResult};
use nest_http_client::{HttpClientService, HTTP_CLIENT_MODULE_ID};

use crate::client::AirtableClient;
use crate::config::AirtableConfig;

/// Module id for [`AirtableModule`].
pub const AIRTABLE_MODULE_ID: ModuleId = ModuleId("nest-airtable");

/// Registers [`AirtableClient`] using configuration and the shared HTTP client.
pub struct AirtableModule {
    config: Option<AirtableConfig>,
}

impl AirtableModule {
    /// Creates a module that loads `[airtable]` from [`ConfigService`] at configure time.
    pub fn new() -> Self {
        Self { config: None }
    }

    /// Creates a module with an explicit configuration.
    pub fn with_config(config: AirtableConfig) -> Self {
        Self {
            config: Some(config),
        }
    }
}

impl Default for AirtableModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for AirtableModule {
    fn id(&self) -> ModuleId {
        AIRTABLE_MODULE_ID
    }

    fn dependencies(&self) -> &'static [ModuleId] {
        &[HTTP_CLIENT_MODULE_ID]
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        let config = match &self.config {
            Some(config) => config.clone(),
            None => {
                let config_service = app.service_mut::<ConfigService>()?;
                AirtableConfig::from_config_service(config_service)?
            }
        };

        let http = app.service_mut::<HttpClientService>()?.clone();
        let client = AirtableClient::new(http, config)?;
        app.register_service(client)
    }
}

#[cfg(test)]
mod tests {
    use nest_config::{ConfigDocument, ConfigService, LoadedConfig};
    use nest_core::AppBuilder;
    use nest_http_client::HttpClientModule;

    use super::*;
    use crate::config::AirtableConfig;

    fn test_config() -> AirtableConfig {
        AirtableConfig::builder("appTEST", "pat-test")
            .table("assets", "tblASSETS", Some("Asset ID".into()))
            .build()
            .unwrap()
    }

    #[test]
    fn module_registers_airtable_client_with_explicit_config() {
        let built = AppBuilder::new()
            .module(HttpClientModule::default())
            .module(AirtableModule::with_config(test_config()))
            .build()
            .unwrap();
        assert!(built.context.has_service::<AirtableClient>());
    }

    #[test]
    fn module_loads_config_from_config_service() {
        std::env::set_var("AIRTABLE_TOKEN", "pat-test");
        let document = ConfigDocument::parse_toml(
            r#"
[airtable]
base_id = "appTEST"

[airtable.tables.assets]
table_id = "tblASSETS"
"#,
        )
        .unwrap();
        let loaded = LoadedConfig {
            document: document.clone(),
            source: nest_config::ConfigSource::Memory(document),
            path: None,
        };

        let mut builder = AppBuilder::new();
        builder
            .register_service(ConfigService::new(loaded))
            .unwrap();
        let built = builder
            .module(HttpClientModule::default())
            .module(AirtableModule::new())
            .build()
            .unwrap();
        assert!(built.context.has_service::<AirtableClient>());
    }

    #[test]
    fn missing_http_client_dependency_fails() {
        let result = AppBuilder::new()
            .module(AirtableModule::with_config(test_config()))
            .build();
        assert_eq!(
            result.err().unwrap().code(),
            Some(nest_error::codes::NEST_MODULE_DEPENDENCY_MISSING)
        );
    }
}
