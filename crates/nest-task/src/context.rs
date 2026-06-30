//! Per-task execution context.

use std::sync::Arc;

use nest_core::AppContext;
use tracing::Span;

use crate::cancel::CancelToken;
use crate::id::TaskId;
use crate::progress::ProgressReporter;

/// Context passed to a task during execution.
#[derive(Clone)]
pub struct TaskContext {
    task_id: TaskId,
    app: Arc<AppContext>,
    cancel: CancelToken,
    progress: ProgressReporter,
    span: Span,
}

impl TaskContext {
    /// Creates a new task context.
    pub fn new(
        task_id: TaskId,
        app: Arc<AppContext>,
        cancel: CancelToken,
        progress: ProgressReporter,
        span: Span,
    ) -> Self {
        Self {
            task_id,
            app,
            cancel,
            progress,
            span,
        }
    }

    /// Returns the task id.
    pub fn task_id(&self) -> &TaskId {
        &self.task_id
    }

    /// Returns the application context.
    pub fn app(&self) -> &AppContext {
        &self.app
    }

    /// Returns the cancellation token.
    pub fn cancel_token(&self) -> &CancelToken {
        &self.cancel
    }

    /// Returns the progress reporter.
    pub fn progress(&self) -> &ProgressReporter {
        &self.progress
    }

    /// Returns the tracing span for this task.
    pub fn span(&self) -> &Span {
        &self.span
    }
}
