//! Task manager contract.

use async_trait::async_trait;
use nest_error::NestResult;

use crate::handle::TaskHandle;
use crate::task::Task;

/// Schedules and tracks background tasks.
#[async_trait]
pub trait TaskManager: Send + Sync + 'static {
    /// Spawns an asynchronous task.
    async fn spawn<T>(&self, task: T) -> NestResult<TaskHandle<T::Output>>
    where
        T: Task;

    /// Spawns a blocking task on the runtime's blocking thread pool.
    async fn spawn_blocking<F, R>(&self, name: &'static str, f: F) -> NestResult<TaskHandle<R>>
    where
        F: FnOnce() -> NestResult<R> + Send + 'static,
        R: Send + Sync + 'static;
}
