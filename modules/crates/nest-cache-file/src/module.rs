//! Nest module registration for the file cache adapter.

use std::sync::Arc;

use nest_cache::{Cache, CacheAdapter};
use nest_core::{AppBuilder, Module, ModuleId, NestResult};

use crate::adapter::FileCacheAdapter;
use crate::config::FileCacheConfig;

/// Module id for [`FileCacheModule`].
pub const FILE_CACHE_MODULE_ID: ModuleId = ModuleId("nest-cache-file");

/// Registers a [`Cache`] service backed by [`FileCacheAdapter`].
pub struct FileCacheModule {
    config: FileCacheConfig,
}

impl FileCacheModule {
    /// Creates a module with explicit configuration.
    pub fn new(config: FileCacheConfig) -> Self {
        Self { config }
    }
}

impl Module for FileCacheModule {
    fn id(&self) -> ModuleId {
        FILE_CACHE_MODULE_ID
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        let adapter: Arc<dyn CacheAdapter> = Arc::new(
            FileCacheAdapter::new(self.config.clone()).map_err(nest_error::NestError::from)?,
        );
        app.register_service(Cache::new(adapter))
    }
}

#[cfg(test)]
mod tests {
    use nest_core::AppBuilder;
    use tempfile::tempdir;

    use super::*;
    use crate::config::FileCacheConfig;

    #[test]
    fn module_registers_cache_service() {
        let dir = tempdir().unwrap();
        let built = AppBuilder::new()
            .module(FileCacheModule::new(FileCacheConfig::new(dir.path())))
            .build()
            .unwrap();
        assert!(built.context.has_service::<Cache>());
    }
}
