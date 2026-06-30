//! Tokio runtime ownership and handle access.

use std::sync::Arc;

use nest_error::NestResult;
use nest_task::TaskError;
use tokio::runtime::{Handle, Runtime};

use crate::config::RuntimeConfig;

struct TaskRuntimeInner {
    #[allow(dead_code)]
    owned: Option<Runtime>,
    handle: Handle,
}

/// Nest task runtime — owns Tokio or adopts an existing handle.
#[derive(Clone)]
pub struct TaskRuntime {
    inner: Arc<TaskRuntimeInner>,
}

impl TaskRuntime {
    /// Creates and owns a new multi-thread Tokio runtime.
    pub fn new_owned(config: RuntimeConfig) -> NestResult<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(config.worker_threads)
            .thread_name(config.thread_name_prefix)
            .enable_all()
            .build()
            .map_err(|error| TaskError::runtime_missing(error.to_string()))?;

        let handle = runtime.handle().clone();
        Ok(Self {
            inner: Arc::new(TaskRuntimeInner {
                owned: Some(runtime),
                handle,
            }),
        })
    }

    /// Adopts an existing Tokio runtime handle.
    pub fn from_handle(handle: Handle) -> Self {
        Self {
            inner: Arc::new(TaskRuntimeInner {
                owned: None,
                handle,
            }),
        }
    }

    /// Adopts the current thread's Tokio runtime handle.
    pub fn from_current() -> NestResult<Self> {
        Handle::try_current()
            .map(Self::from_handle)
            .map_err(|error| TaskError::runtime_missing(error.to_string()).into())
    }

    /// Returns the Tokio runtime handle.
    pub fn handle(&self) -> &Handle {
        &self.inner.handle
    }
}
