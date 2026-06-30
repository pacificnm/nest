//! Task handle backend and public handle API.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use nest_error::NestResult;

use crate::id::TaskId;
use crate::progress::TaskProgress;
use crate::status::TaskStatus;

/// Runtime-specific backend for [`TaskHandle`].
pub trait TaskHandleBackend<O>: Send + Sync
where
    O: Send + Sync + 'static,
{
    /// Returns the task id.
    fn id(&self) -> TaskId;

    /// Returns the current status.
    fn status(&self) -> TaskStatus;

    /// Returns the current progress snapshot.
    fn progress(&self) -> TaskProgress;

    /// Requests cancellation.
    fn cancel(&self);

    /// Returns whether the task has finished.
    fn is_finished(&self) -> bool;

    /// Waits for the task result.
    fn await_result(&self) -> Pin<Box<dyn Future<Output = NestResult<O>> + Send + '_>>;
}

/// Handle to an in-flight or completed task.
pub struct TaskHandle<O>
where
    O: Send + Sync + 'static,
{
    inner: Arc<dyn TaskHandleBackend<O>>,
}

impl<O> TaskHandle<O>
where
    O: Send + Sync + 'static,
{
    /// Creates a handle from a runtime backend.
    pub fn new(inner: Arc<dyn TaskHandleBackend<O>>) -> Self {
        Self { inner }
    }

    /// Returns the task id.
    pub fn id(&self) -> TaskId {
        self.inner.id()
    }

    /// Returns the current status.
    pub fn status(&self) -> TaskStatus {
        self.inner.status()
    }

    /// Returns the current progress snapshot.
    pub fn progress(&self) -> TaskProgress {
        self.inner.progress()
    }

    /// Requests cancellation.
    pub fn cancel(&self) {
        self.inner.cancel();
    }

    /// Returns whether the task has finished.
    pub fn is_finished(&self) -> bool {
        self.inner.is_finished()
    }

    /// Waits for the task to complete and returns its output.
    pub async fn await_result(&self) -> NestResult<O> {
        self.inner.await_result().await
    }
}
