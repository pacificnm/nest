//! Registers an [`OAuthClient`] service.

use nest_config::ConfigService;
use nest_core::{AppBuilder, Module, ModuleId, NestResult};
use nest_error::NestError;
use nest_http_client::HTTP_CLIENT_MODULE_ID;

use crate::client::OAuthClient;
use crate::config::OAuthClientConfig;

/// Module id for [`OAuthClientModule`].
pub const OAUTH_CLIENT_MODULE_ID: ModuleId = ModuleId("nest-auth-oauth-client");

/// Registers an [`OAuthClient`] built from `[oauth_client]` config (or an
/// explicit [`OAuthClientConfig`]).
///
/// Only registers the client itself — not a [`nest_auth::TokenStore`] (app
/// code chooses and constructs its own, same as `nest-auth` doesn't ship a
/// default `Module`) and not a [`crate::OAuthTokenAuth`] (nothing to
/// authenticate with until a login actually completes).
pub struct OAuthClientModule {
    config: Option<OAuthClientConfig>,
}

impl OAuthClientModule {
    /// Creates a module that loads `[oauth_client]` from [`ConfigService`].
    pub fn new() -> Self {
        Self { config: None }
    }

    /// Creates a module with an explicit configuration.
    pub fn with_config(config: OAuthClientConfig) -> Self {
        Self {
            config: Some(config),
        }
    }
}

impl Default for OAuthClientModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for OAuthClientModule {
    fn id(&self) -> ModuleId {
        OAUTH_CLIENT_MODULE_ID
    }

    fn dependencies(&self) -> &'static [ModuleId] {
        &[HTTP_CLIENT_MODULE_ID]
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        let config = match &self.config {
            Some(config) => config.clone(),
            None => {
                let config_service = app.service_mut::<ConfigService>()?;
                OAuthClientConfig::from_config_service(config_service)?.ok_or_else(|| {
                    NestError::config("no [oauth_client] section configured")
                        .with_module("nest-auth-oauth-client")
                })?
            }
        };

        let client = OAuthClient::new(&config)?;
        app.register_service(client)
    }
}

#[cfg(test)]
mod tests {
    use nest_config::{ConfigDocument, ConfigService, LoadedConfig};
    use nest_core::AppBuilder;
    use nest_http_client::HttpClientModule;

    use super::*;

    #[test]
    fn module_registers_oauth_client_with_explicit_config() {
        let config = OAuthClientConfig::new(
            "client-id",
            "https://example.com/authorize",
            "https://example.com/token",
        );
        let built = AppBuilder::new()
            .module(HttpClientModule::default())
            .module(OAuthClientModule::with_config(config))
            .build()
            .unwrap();
        assert!(built.context.has_service::<OAuthClient>());
    }

    #[test]
    fn module_loads_oauth_client_section_from_config() {
        let document = ConfigDocument::parse_toml(
            r#"
[oauth_client]
client_id = "client-id"
auth_url = "https://example.com/authorize"
token_url = "https://example.com/token"
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
            .module(OAuthClientModule::new())
            .build()
            .unwrap();
        assert!(built.context.has_service::<OAuthClient>());
    }
}
