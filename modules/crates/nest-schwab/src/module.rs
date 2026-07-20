//! Registers a Schwab-configured [`OAuthClient`].

use nest_auth_oauth_client::OAuthClient;
use nest_config::ConfigService;
use nest_core::{AppBuilder, Module, ModuleId, NestResult};
use nest_error::NestError;
use nest_http_client::HTTP_CLIENT_MODULE_ID;

use crate::config::SchwabConfig;

/// Module id for [`SchwabModule`].
pub const SCHWAB_MODULE_ID: ModuleId = ModuleId("nest-schwab");

/// Registers an [`OAuthClient`] pre-configured with Schwab's real
/// endpoints, scope, and HTTPS-callback requirement, built from `[schwab]`
/// config (or an explicit [`SchwabConfig`]).
///
/// Only registers the `OAuthClient` — not a [`crate::SchwabClient`], which
/// needs a live [`nest_auth::Token`] that doesn't exist until a login
/// actually completes (mirrors `nest-auth-oauth-client`'s own
/// `OAuthClientModule` for the same reason).
pub struct SchwabModule {
    config: Option<SchwabConfig>,
}

impl SchwabModule {
    /// Creates a module that loads `[schwab]` from [`ConfigService`].
    pub fn new() -> Self {
        Self { config: None }
    }

    /// Creates a module with an explicit configuration.
    pub fn with_config(config: SchwabConfig) -> Self {
        Self {
            config: Some(config),
        }
    }
}

impl Default for SchwabModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for SchwabModule {
    fn id(&self) -> ModuleId {
        SCHWAB_MODULE_ID
    }

    fn dependencies(&self) -> &'static [ModuleId] {
        &[HTTP_CLIENT_MODULE_ID]
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        let config = match &self.config {
            Some(config) => config.clone(),
            None => {
                let config_service = app.service_mut::<ConfigService>()?;
                SchwabConfig::from_config_service(config_service)?.ok_or_else(|| {
                    NestError::config("no [schwab] section configured").with_module("nest-schwab")
                })?
            }
        };

        let oauth_client = OAuthClient::new(&config.to_oauth_client_config())?;
        app.register_service(config)?;
        app.register_service(oauth_client)
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
        let config = SchwabConfig::new("app-key", "app-secret");
        let built = AppBuilder::new()
            .module(HttpClientModule::default())
            .module(SchwabModule::with_config(config))
            .build()
            .unwrap();
        assert!(built.context.has_service::<OAuthClient>());
        assert!(built.context.has_service::<SchwabConfig>());
    }

    #[test]
    fn module_loads_schwab_section_from_config() {
        let document = ConfigDocument::parse_toml(
            r#"
[schwab]
app_key = "app-key"
app_secret = "app-secret"
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
            .module(SchwabModule::new())
            .build()
            .unwrap();
        assert!(built.context.has_service::<OAuthClient>());
    }
}
