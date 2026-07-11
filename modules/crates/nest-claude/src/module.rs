//! Registers [`crate::ClaudeClient`] via Nest module configuration.

use nest_core::{AppBuilder, Module, ModuleId, NestResult};

use crate::client::ClaudeClient;
use crate::config::ClaudeConfig;

/// Module id for [`ClaudeModule`].
pub const CLAUDE_MODULE_ID: ModuleId = ModuleId("nest-claude");

/// Registers a [`ClaudeClient`] built from explicit config or `[claude]` in
/// [`nest_config::ConfigService`].
pub struct ClaudeModule {
    config: Option<ClaudeConfig>,
}

impl ClaudeModule {
    /// Creates a module that loads `[claude]` from [`nest_config::ConfigService`] at configure time.
    pub fn new() -> Self {
        Self { config: None }
    }

    /// Creates a module with an explicit configuration.
    pub fn with_config(config: ClaudeConfig) -> Self {
        Self {
            config: Some(config),
        }
    }
}

impl Default for ClaudeModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for ClaudeModule {
    fn id(&self) -> ModuleId {
        CLAUDE_MODULE_ID
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        let config = match &self.config {
            Some(config) => config.clone(),
            None => {
                #[cfg(feature = "config")]
                {
                    let config_service = app.service_mut::<nest_config::ConfigService>()?;
                    ClaudeConfig::from_config_service(config_service)?
                }
                #[cfg(not(feature = "config"))]
                {
                    return Err(nest_error::NestError::config(
                        "ClaudeModule requires explicit config when nest-config feature is disabled",
                    )
                    .with_module("nest-claude"));
                }
            }
        };

        let client = ClaudeClient::new(config).map_err(nest_error::NestError::from)?;
        app.register_service(client)
    }
}

#[cfg(test)]
mod tests {
    use nest_core::AppBuilder;

    use super::*;

    fn test_config() -> ClaudeConfig {
        ClaudeConfig::builder().api_key("test-key").build().unwrap()
    }

    #[test]
    fn module_registers_claude_client() {
        let built = AppBuilder::new()
            .module(ClaudeModule::with_config(test_config()))
            .build()
            .unwrap();
        assert!(built.context.has_service::<ClaudeClient>());
    }
}
