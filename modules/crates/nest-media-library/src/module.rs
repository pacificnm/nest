//! Registers media library services via Nest module configuration.

use std::sync::Arc;

use nest_core::{AppBuilder, Module, ModuleId, NestResult};
use nest_file::{FileService, FILE_MODULE_ID};
use nest_media::{MediaInspector, MediaLibraryRepository, MetadataProvider};

use crate::indexer::LibraryIndexer;
use crate::scan::LibraryScanner;

/// Module id for [`MediaLibraryModule`].
pub const MEDIA_LIBRARY_MODULE_ID: ModuleId = ModuleId("nest-media-library");

/// Registers [`LibraryScanner`] and [`LibraryIndexer`].
pub struct MediaLibraryModule {
    metadata: Option<Arc<dyn MetadataProvider>>,
    inspector: Option<Arc<dyn MediaInspector>>,
    repository: Option<Arc<dyn MediaLibraryRepository>>,
}

impl MediaLibraryModule {
    /// Creates a scan-only module with no injected providers.
    pub fn new() -> Self {
        Self {
            metadata: None,
            inspector: None,
            repository: None,
        }
    }

    /// Attaches a metadata provider for indexing.
    pub fn with_metadata(mut self, metadata: Arc<dyn MetadataProvider>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Attaches a media inspector for indexing.
    pub fn with_inspector(mut self, inspector: Arc<dyn MediaInspector>) -> Self {
        self.inspector = Some(inspector);
        self
    }

    /// Attaches a media library repository for indexing.
    pub fn with_repository(mut self, repository: Arc<dyn MediaLibraryRepository>) -> Self {
        self.repository = Some(repository);
        self
    }
}

impl Default for MediaLibraryModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for MediaLibraryModule {
    fn id(&self) -> ModuleId {
        MEDIA_LIBRARY_MODULE_ID
    }

    fn dependencies(&self) -> &'static [ModuleId] {
        &[FILE_MODULE_ID]
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        let files = app.service_mut::<FileService>()?.clone();
        let scanner = LibraryScanner::new(files);
        app.register_service(scanner.clone())?;

        let mut indexer = LibraryIndexer::new(scanner);
        if let Some(metadata) = &self.metadata {
            indexer = indexer.with_metadata(metadata.clone());
        }
        if let Some(inspector) = &self.inspector {
            indexer = indexer.with_inspector(inspector.clone());
        }
        if let Some(repository) = &self.repository {
            indexer = indexer.with_repository(repository.clone());
        }

        app.register_service(indexer)
    }
}

#[cfg(test)]
mod tests {
    use nest_core::AppBuilder;
    use nest_file::FileModule;
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn module_registers_scanner_and_indexer() {
        let dir = tempdir().unwrap();
        let built = AppBuilder::new()
            .module(FileModule::scoped(dir.path()))
            .module(MediaLibraryModule::new())
            .build()
            .unwrap();

        assert!(built.context.has_service::<LibraryScanner>());
        assert!(built.context.has_service::<LibraryIndexer>());
    }

    #[test]
    fn missing_file_dependency_fails() {
        let result = AppBuilder::new().module(MediaLibraryModule::new()).build();
        assert_eq!(
            result.err().unwrap().code(),
            Some(nest_error::codes::NEST_MODULE_DEPENDENCY_MISSING)
        );
    }
}
