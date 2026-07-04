//! Nest module that registers [`crate::IconService`].

use nest_core::{AppBuilder, Module, ModuleId, NestResult};

use crate::service::IconService;

/// Module id for [`IconModule`].
pub const ICON_MODULE_ID: ModuleId = ModuleId("nest-icon");

/// Registers [`IconService`] for Font Awesome icon fonts.
pub struct IconModule;

impl IconModule {
    /// Creates the icon module.
    pub fn new() -> Self {
        Self
    }
}

impl Default for IconModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for IconModule {
    fn id(&self) -> ModuleId {
        ICON_MODULE_ID
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        app.register_service(IconService::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font;
    use crate::IconService;
    use nest_core::AppBuilder;

    #[test]
    fn module_registers_icon_service() {
        let built = AppBuilder::new().module(IconModule::new()).build().unwrap();
        let _icons = built.context.service::<IconService>().unwrap();
        font::install(&egui::Context::default());
    }
}
