//! Built Nest application container.

use std::sync::Arc;

use nest_core::{AppContext, BuiltApp};

use crate::lifecycle::AppLifecycleRunner;
use crate::metadata::AppMetadata;
use nest_error::NestResult;

/// Host-agnostic Nest application container.
pub struct NestApp {
    metadata: AppMetadata,
    built: BuiltApp,
    started: bool,
}

impl NestApp {
    /// Creates a container from metadata and a built core application.
    pub(crate) fn new(metadata: AppMetadata, built: BuiltApp) -> Self {
        Self {
            metadata,
            built,
            started: false,
        }
    }

    /// Returns application metadata.
    pub fn metadata(&self) -> &AppMetadata {
        &self.metadata
    }

    /// Returns the shared application context.
    pub fn context(&self) -> &AppContext {
        self.built.context.as_ref()
    }

    /// Returns a clone of the shared application context handle.
    pub fn context_arc(&self) -> Arc<AppContext> {
        Arc::clone(&self.built.context)
    }

    /// Returns whether startup lifecycle hooks have run.
    pub fn is_started(&self) -> bool {
        self.started
    }

    /// Runs startup lifecycle hooks.
    pub fn startup(&mut self) -> NestResult<()> {
        AppLifecycleRunner::startup(self)
    }

    /// Runs shutdown lifecycle hooks (idempotent when not started).
    pub fn shutdown(&mut self) -> NestResult<()> {
        AppLifecycleRunner::shutdown(self)
    }

    /// Consumes the container and returns the underlying [`BuiltApp`].
    pub fn into_built(mut self) -> BuiltApp {
        let _ = AppLifecycleRunner::shutdown(&mut self);
        self.built
    }

    pub(crate) fn built_mut(&mut self) -> &mut BuiltApp {
        &mut self.built
    }

    pub(crate) fn set_started(&mut self, started: bool) {
        self.started = started;
    }
}
