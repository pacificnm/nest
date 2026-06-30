//! Optional Nest module that registers [`crate::DataService`].

use nest_core::{AppBuilder, Module, ModuleId, NestResult};

use crate::service::DataService;

/// Module id for [`DataModule`].
pub const DATA_MODULE_ID: ModuleId = ModuleId("nest-data");

/// Registers [`DataService`].
pub struct DataModule;

impl Module for DataModule {
    fn id(&self) -> ModuleId {
        DATA_MODULE_ID
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        app.register_service(DataService::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nest_core::AppBuilder;

    #[test]
    fn module_registers_data_service() {
        let built = AppBuilder::new().module(DataModule).build().unwrap();
        assert!(built.context.has_service::<DataService>());
    }
}
