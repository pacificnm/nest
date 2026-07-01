//! Registers transcode services via Nest module configuration.

use nest_core::{AppBuilder, Module, ModuleId, NestResult};
use nest_file::{FileService, FILE_MODULE_ID};

use crate::config::TranscodeConfig;
use crate::inspector::FfprobeMediaInspector;
use crate::runner::FfprobeRunner;

/// Module id for [`TranscodeModule`].
pub const TRANSCODE_MODULE_ID: ModuleId = ModuleId("nest-transcode");

/// Registers [`FfprobeRunner`] and [`FfprobeMediaInspector`].
pub struct TranscodeModule {
    config: Option<TranscodeConfig>,
}

impl TranscodeModule {
    /// Creates a module that loads `[transcode]` from [`nest_config::ConfigService`] at configure time.
    pub fn new() -> Self {
        Self { config: None }
    }

    /// Creates a module with an explicit configuration.
    pub fn with_config(config: TranscodeConfig) -> Self {
        Self {
            config: Some(config),
        }
    }
}

impl Default for TranscodeModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for TranscodeModule {
    fn id(&self) -> ModuleId {
        TRANSCODE_MODULE_ID
    }

    fn dependencies(&self) -> &'static [ModuleId] {
        &[FILE_MODULE_ID]
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        let config = match &self.config {
            Some(config) => config.clone(),
            None => {
                #[cfg(feature = "config")]
                {
                    let config_service = app.service_mut::<nest_config::ConfigService>()?;
                    TranscodeConfig::from_config_service(config_service)?
                }
                #[cfg(not(feature = "config"))]
                {
                    return Err(nest_error::NestError::config(
                        "TranscodeModule requires explicit config when nest-config feature is disabled",
                    )
                    .with_module("nest-transcode"));
                }
            }
        };

        let files = app.service_mut::<FileService>()?.clone();
        let runner = FfprobeRunner::new(config)?;
        let inspector = FfprobeMediaInspector::new(files, runner.clone());

        app.register_service(runner)?;
        app.register_service(inspector)
    }
}

#[cfg(test)]
mod tests {
    use nest_core::AppBuilder;
    use nest_file::FileModule;
    use tempfile::tempdir;

    use super::*;
    use crate::config::TranscodeConfig;

    fn test_config() -> TranscodeConfig {
        TranscodeConfig::builder().build().unwrap()
    }

    #[test]
    fn module_registers_transcode_services() {
        let dir = tempdir().unwrap();
        let built = AppBuilder::new()
            .module(FileModule::scoped(dir.path()))
            .module(TranscodeModule::with_config(test_config()))
            .build()
            .unwrap();

        assert!(built.context.has_service::<FfprobeRunner>());
        assert!(built.context.has_service::<FfprobeMediaInspector>());
    }

    #[test]
    fn missing_file_dependency_fails() {
        let result = AppBuilder::new()
            .module(TranscodeModule::with_config(test_config()))
            .build();
        assert_eq!(
            result.err().unwrap().code(),
            Some(nest_error::codes::NEST_MODULE_DEPENDENCY_MISSING)
        );
    }
}
