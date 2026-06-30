//! Optional Nest module that registers [`crate::service::CsvService`].

use nest_core::{AppBuilder, Module, ModuleId, NestResult};
use nest_file::{FileService, FILE_MODULE_ID};

use crate::service::CsvService;

/// Module id for [`CsvModule`].
pub const CSV_MODULE_ID: ModuleId = ModuleId("nest-file-csv");

/// Registers [`CsvService`], depending on [`nest_file::FileModule`].
pub struct CsvModule;

impl Default for CsvModule {
    fn default() -> Self {
        Self
    }
}

impl Module for CsvModule {
    fn id(&self) -> ModuleId {
        CSV_MODULE_ID
    }

    fn dependencies(&self) -> &'static [ModuleId] {
        &[FILE_MODULE_ID]
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        let files = app.service_mut::<FileService>()?.clone();
        app.register_service(CsvService::new(files))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nest_core::AppBuilder;
    use nest_file::FileModule;

    #[test]
    fn module_registers_csv_service() {
        let built = AppBuilder::new()
            .module(FileModule::default())
            .module(CsvModule)
            .build()
            .unwrap();
        assert!(built.context.has_service::<CsvService>());
    }
}
