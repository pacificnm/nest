//! Background library scan task.

use async_trait::async_trait;
use nest_error::NestResult;
use nest_task::{Task, TaskContext};

use crate::config::MediaLibraryConfig;
use crate::indexer::LibraryIndexer;
use crate::scan::{LibraryScanOptions, ScanResult};

/// Runs a library scan through [`LibraryIndexer`].
pub struct LibraryScanTask {
    config: MediaLibraryConfig,
    options: LibraryScanOptions,
}

impl LibraryScanTask {
    /// Creates a discovery-only scan task.
    pub fn discover(config: MediaLibraryConfig) -> Self {
        Self {
            config,
            options: LibraryScanOptions::discover_only(),
        }
    }

    /// Creates a scan task with explicit options.
    pub fn with_options(config: MediaLibraryConfig, options: LibraryScanOptions) -> Self {
        Self { config, options }
    }
}

#[async_trait]
impl Task for LibraryScanTask {
    type Output = ScanResult;

    fn name(&self) -> &'static str {
        "library-scan"
    }

    async fn run(&self, ctx: TaskContext) -> NestResult<Self::Output> {
        let indexer = ctx.app().service::<LibraryIndexer>()?;
        ctx.progress()
            .set_message(format!("scanning library {}", self.config.id));
        indexer
            .scan_library(&self.config, self.options)
            .await
            .map_err(Into::into)
    }
}
