//! Registers [`nest_ai::AiService`] backed by [`crate::OllamaProvider`].

use std::sync::Arc;

use nest_config::ConfigService;
use nest_core::{AppBuilder, Module, ModuleId, NestResult};
use nest_http_client::{HttpClientService, HTTP_CLIENT_MODULE_ID};

use crate::config::OllamaConfig;
use crate::provider::OllamaProvider;
use crate::shared::OllamaSharedConfig;
use nest_ai::AiService;

/// Module id for [`OllamaModule`].
pub const OLLAMA_MODULE_ID: ModuleId = ModuleId("nest-ai-ollama");

/// Registers an Ollama-backed [`AiService`].
pub struct OllamaModule {
    config: Option<OllamaConfig>,
}

impl OllamaModule {
    /// Creates a module that loads `[ai]` / `[ollama]` from [`ConfigService`].
    pub fn new() -> Self {
        Self { config: None }
    }

    /// Creates a module with an explicit configuration.
    pub fn with_config(config: OllamaConfig) -> Self {
        Self {
            config: Some(config),
        }
    }
}

impl Default for OllamaModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for OllamaModule {
    fn id(&self) -> ModuleId {
        OLLAMA_MODULE_ID
    }

    fn dependencies(&self) -> &'static [ModuleId] {
        &[HTTP_CLIENT_MODULE_ID]
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        let config = match &self.config {
            Some(config) => config.clone(),
            None => {
                let config_service = app.service_mut::<ConfigService>()?;
                OllamaConfig::from_config_service(config_service)?
                    .unwrap_or_else(OllamaConfig::default_local)
            }
        };

        let http = app.service_mut::<HttpClientService>()?.clone();
        let shared = OllamaSharedConfig::new(config);
        let provider = OllamaProvider::with_shared_config(http, shared.clone())?;
        app.register_service(shared)?;
        app.register_service(AiService::new(Arc::new(provider)))
    }
}

#[cfg(test)]
mod tests {
    use nest_config::{ConfigDocument, ConfigService, LoadedConfig};
    use nest_core::AppBuilder;
    use nest_http_client::HttpClientModule;

    use super::*;
    use crate::config::OllamaConfig;

    #[test]
    fn module_registers_ai_service_with_explicit_config() {
        let config = OllamaConfig::new("http://127.0.0.1:11434", "test-model");
        let built = AppBuilder::new()
            .module(HttpClientModule::default())
            .module(OllamaModule::with_config(config))
            .build()
            .unwrap();
        assert!(built.context.has_service::<AiService>());
    }

    #[test]
    fn module_loads_ai_section_from_config() {
        let document = ConfigDocument::parse_toml(
            r#"
[ai]
enabled = true
provider = "ollama"
base_url = "http://127.0.0.1:11434"
model = "llama3"
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
            .module(OllamaModule::new())
            .build()
            .unwrap();
        assert!(built.context.has_service::<AiService>());
    }
}
