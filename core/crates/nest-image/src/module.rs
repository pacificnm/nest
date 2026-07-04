//! Nest module registration for [`crate::ImageService`].

use nest_cache::Cache;
use nest_core::{AppBuilder, Module, ModuleId, NestResult};

use crate::service::ImageService;

/// Module id for [`ImageModule`].
pub const IMAGE_MODULE_ID: ModuleId = ModuleId("nest-image");

/// Registers [`ImageService`] using an existing [`Cache`] instance.
///
/// Register a file or memory cache first (e.g. [`nest_cache_file::FileCacheModule`]),
/// or use [`ImageModule::with_cache`] from app wiring that creates both.
pub struct ImageModule {
    cache: Cache,
}

impl ImageModule {
    /// Creates a module that registers [`ImageService`] over `cache`.
    pub fn with_cache(cache: Cache) -> Self {
        Self { cache }
    }
}

impl Module for ImageModule {
    fn id(&self) -> ModuleId {
        IMAGE_MODULE_ID
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        app.register_service(ImageService::new(self.cache.clone())?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nest_cache::Cache;
    use nest_core::AppBuilder;

    #[test]
    fn module_registers_image_service() {
        let cache = Cache::memory();
        let built = AppBuilder::new()
            .module(ImageModule::with_cache(cache))
            .build()
            .unwrap();
        assert!(built.context.has_service::<ImageService>());
    }
}
