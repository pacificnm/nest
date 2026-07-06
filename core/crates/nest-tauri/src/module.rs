//! Host module marker for Tauri applications.

use nest_core::{AppBuilder, Module, ModuleId, NestResult};

/// Module id for the Tauri host.
pub const TAURI_MODULE_ID: ModuleId = ModuleId("nest-tauri");

/// Registers the Tauri host module identity in the module graph.
pub struct TauriModule;

impl Module for TauriModule {
    fn id(&self) -> ModuleId {
        TAURI_MODULE_ID
    }

    fn configure(&self, _app: &mut AppBuilder) -> NestResult<()> {
        Ok(())
    }
}
