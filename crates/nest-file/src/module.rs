//! Optional Nest module that registers [`crate::service::FileService`].

use nest_core::{AppBuilder, Module, ModuleId, NestResult};

use crate::config::FileServiceConfig;
use crate::service::FileService;

/// Module id for [`FileModule`].
pub const FILE_MODULE_ID: ModuleId = ModuleId("nest-file");

/// Registers [`FileService`].
pub struct FileModule {
    config: FileServiceConfig,
}

impl FileModule {
    /// Creates a module with default (unscoped) configuration.
    pub fn new() -> Self {
        Self {
            config: FileServiceConfig::default(),
        }
    }

    /// Creates a module with scoped root configuration.
    pub fn scoped(root: impl Into<std::path::PathBuf>) -> Self {
        Self {
            config: FileServiceConfig::scoped(root),
        }
    }

    /// Creates a module with custom configuration.
    pub fn with_config(config: FileServiceConfig) -> Self {
        Self { config }
    }
}

impl Default for FileModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for FileModule {
    fn id(&self) -> ModuleId {
        FILE_MODULE_ID
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        let service = FileService::with_config(self.config.clone())?;
        app.register_service(service)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nest_core::AppBuilder;

    #[test]
    fn module_registers_file_service() {
        let built = AppBuilder::new().module(FileModule::default()).build().unwrap();
        assert!(built.context.has_service::<FileService>());
    }
}
