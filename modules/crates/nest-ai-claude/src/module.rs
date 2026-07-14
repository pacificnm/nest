//! `ClaudeAiModule`: registers an `nest_ai::AiService` backed by `ClaudeAiProvider`.

use std::sync::Arc;

use nest_ai::AiService;
use nest_claude::ClaudeConfig;
use nest_config::ConfigService;
use nest_core::{AppBuilder, Module, ModuleId, NestResult};

use crate::provider::ClaudeAiProvider;

/// Module id for [`ClaudeAiModule`].
pub const CLAUDE_AI_MODULE_ID: ModuleId = ModuleId("nest-ai-claude");

/// Registers a Claude-backed [`AiService`].
///
/// Reads the `[claude]` config section via [`ClaudeConfig::from_config_service`]
/// when no explicit config is given — the **same** section `nest-claude`'s own
/// `ClaudeModule` reads. Registering both `ClaudeAiModule` and `ClaudeModule`
/// in the same app is fine and intentional (one API key, one set of
/// defaults), just not obvious from the module name alone.
pub struct ClaudeAiModule {
    config: Option<ClaudeConfig>,
}

impl ClaudeAiModule {
    /// Creates a module that loads `[claude]` from [`ConfigService`].
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

impl Default for ClaudeAiModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for ClaudeAiModule {
    fn id(&self) -> ModuleId {
        CLAUDE_AI_MODULE_ID
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        let config = match &self.config {
            Some(config) => config.clone(),
            None => {
                let config_service = app.service_mut::<ConfigService>()?;
                ClaudeConfig::from_config_service(config_service)?
            }
        };

        let provider = ClaudeAiProvider::new(config)?;
        app.register_service(AiService::new(Arc::new(provider)))
    }
}

#[cfg(test)]
mod tests {
    use nest_config::{ConfigDocument, ConfigService, LoadedConfig};
    use nest_core::AppBuilder;

    use super::*;

    #[test]
    fn module_registers_ai_service_with_explicit_config() {
        let config = ClaudeConfig::builder().api_key("test-key").build().unwrap();
        let built = AppBuilder::new()
            .module(ClaudeAiModule::with_config(config))
            .build()
            .unwrap();
        assert!(built.context.has_service::<AiService>());
    }

    #[test]
    fn module_loads_claude_section_from_config() {
        let document = ConfigDocument::parse_toml(
            r#"
[claude]
api_key = "test-key"
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
        let built = builder.module(ClaudeAiModule::new()).build().unwrap();
        assert!(built.context.has_service::<AiService>());
    }
}
