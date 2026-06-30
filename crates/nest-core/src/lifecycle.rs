//! Synchronous application lifecycle hooks.

use std::sync::Arc;

use crate::context::AppContext;
use nest_error::NestResult;

/// Synchronous lifecycle hooks invoked during application startup and shutdown.
///
/// Async lifecycle hooks are provided by `nest-task-runtime` for task shutdown.
pub trait Lifecycle: Send + 'static {
    /// Called after the application is built, before the main loop runs.
    fn on_startup(&mut self, ctx: Arc<AppContext>) -> NestResult<()> {
        let _ = ctx;
        Ok(())
    }

    /// Called when the application is shutting down.
    fn on_shutdown(&mut self, ctx: Arc<AppContext>) -> NestResult<()> {
        let _ = ctx;
        Ok(())
    }
}
