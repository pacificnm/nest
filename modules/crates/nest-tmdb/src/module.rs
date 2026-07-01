//! Registers TMDB services via Nest module configuration.

use nest_core::{AppBuilder, Module, ModuleId, NestResult};
use nest_http_client::{HttpClientService, HTTP_CLIENT_MODULE_ID};

use crate::client::TmdbClient;
use crate::config::TmdbConfig;
use crate::images::TmdbImageService;
use crate::provider::TmdbMetadataProvider;

/// Module id for [`TmdbModule`].
pub const TMDB_MODULE_ID: ModuleId = ModuleId("nest-tmdb");

/// Registers [`TmdbClient`], [`TmdbImageService`], and [`TmdbMetadataProvider`].
pub struct TmdbModule {
    config: Option<TmdbConfig>,
}

impl TmdbModule {
    /// Creates a module that loads `[tmdb]` from [`nest_config::ConfigService`] at configure time.
    pub fn new() -> Self {
        Self { config: None }
    }

    /// Creates a module with an explicit configuration.
    pub fn with_config(config: TmdbConfig) -> Self {
        Self {
            config: Some(config),
        }
    }
}

impl Default for TmdbModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for TmdbModule {
    fn id(&self) -> ModuleId {
        TMDB_MODULE_ID
    }

    fn dependencies(&self) -> &'static [ModuleId] {
        &[HTTP_CLIENT_MODULE_ID]
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        let config = match &self.config {
            Some(config) => config.clone(),
            None => {
                #[cfg(feature = "config")]
                {
                    let config_service = app.service_mut::<nest_config::ConfigService>()?;
                    TmdbConfig::from_config_service(config_service)?
                }
                #[cfg(not(feature = "config"))]
                {
                    return Err(nest_error::NestError::config(
                        "TmdbModule requires explicit config when nest-config feature is disabled",
                    )
                    .with_module("nest-tmdb"));
                }
            }
        };

        let http = app.service_mut::<HttpClientService>()?.clone();
        let client = TmdbClient::new(http, config)?;
        let images = TmdbImageService::new(client.image_base_url());
        let provider = TmdbMetadataProvider::new(client.clone());

        app.register_service(client)?;
        app.register_service(images)?;
        app.register_service(provider)
    }
}

#[cfg(test)]
mod tests {
    use nest_core::AppBuilder;
    use nest_http_client::HttpClientModule;

    use super::*;
    use crate::config::TmdbConfig;

    fn test_config() -> TmdbConfig {
        TmdbConfig::builder().api_key("test-key").build().unwrap()
    }

    #[test]
    fn module_registers_tmdb_services() {
        let built = AppBuilder::new()
            .module(HttpClientModule::default())
            .module(TmdbModule::with_config(test_config()))
            .build()
            .unwrap();

        assert!(built.context.has_service::<TmdbClient>());
        assert!(built.context.has_service::<TmdbImageService>());
        assert!(built.context.has_service::<TmdbMetadataProvider>());
    }

    #[test]
    fn missing_http_client_dependency_fails() {
        let result = AppBuilder::new()
            .module(TmdbModule::with_config(test_config()))
            .build();
        assert_eq!(
            result.err().unwrap().code(),
            Some(nest_error::codes::NEST_MODULE_DEPENDENCY_MISSING)
        );
    }
}
