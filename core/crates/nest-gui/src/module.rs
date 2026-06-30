//! Host module marker for GUI applications.

use nest_core::{AppBuilder, Module, ModuleId, NestResult};

/// Module id for the GUI host.
pub const GUI_MODULE_ID: ModuleId = ModuleId("nest-gui");

/// Registers the GUI host module identity in the module graph.
pub struct GuiModule;

impl Module for GuiModule {
    fn id(&self) -> ModuleId {
        GUI_MODULE_ID
    }

    fn configure(&self, _app: &mut AppBuilder) -> NestResult<()> {
        Ok(())
    }
}
