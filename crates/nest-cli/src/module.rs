//! CLI application module.

use nest_core::{AppBuilder, Module, ModuleId, NestResult};

/// Module id for [`CliModule`].
pub const CLI_MODULE_ID: ModuleId = ModuleId("nest-cli");

/// Nest module marker for CLI-hosted applications.
pub struct CliModule;

impl Default for CliModule {
    fn default() -> Self {
        Self
    }
}

impl Module for CliModule {
    fn id(&self) -> ModuleId {
        CLI_MODULE_ID
    }

    fn configure(&self, _app: &mut AppBuilder) -> NestResult<()> {
        Ok(())
    }
}
