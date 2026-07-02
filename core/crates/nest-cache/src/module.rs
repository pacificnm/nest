//! Nest module registration for [`crate::Cache`].

use std::sync::Arc;

use nest_core::{AppBuilder, Module, ModuleId, NestResult};

use crate::adapter::CacheAdapter;
use crate::cache::Cache;

/// Module id for [`CacheModule`].
pub const CACHE_MODULE_ID: ModuleId = ModuleId("nest-cache");

/// Registers a [`Cache`] service backed by the given adapter.
pub struct CacheModule {
    adapter: Arc<dyn CacheAdapter>,
}

impl CacheModule {
    /// Creates a module with an explicit adapter.
    pub fn new(adapter: Arc<dyn CacheAdapter>) -> Self {
        Self { adapter }
    }

    /// Creates a module with the default in-memory adapter.
    pub fn memory() -> Self {
        Self::new(Arc::new(crate::adapter::MemoryCacheAdapter::new()))
    }
}

impl Module for CacheModule {
    fn id(&self) -> ModuleId {
        CACHE_MODULE_ID
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        app.register_service(Cache::new(self.adapter.clone()))
    }
}

#[cfg(test)]
mod tests {
    use nest_core::AppBuilder;

    use super::*;

    #[test]
    fn module_registers_cache_service() {
        let built = AppBuilder::new()
            .module(CacheModule::memory())
            .build()
            .unwrap();
        assert!(built.context.has_service::<Cache>());
    }
}
