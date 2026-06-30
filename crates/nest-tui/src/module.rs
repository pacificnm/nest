//! Host module marker for TUI applications.

use nest_core::{AppBuilder, Module, ModuleId, NestResult};

/// Module id for the TUI host.
pub const TUI_MODULE_ID: ModuleId = ModuleId("nest-tui");

/// Registers the TUI host module identity in the module graph.
pub struct TuiModule;

impl Module for TuiModule {
    fn id(&self) -> ModuleId {
        TUI_MODULE_ID
    }

    fn configure(&self, _app: &mut AppBuilder) -> NestResult<()> {
        Ok(())
    }
}
