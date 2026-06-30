//! Task lifecycle events.

use crate::id::TaskId;
use crate::progress::TaskProgress;

/// Kind of task lifecycle event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskEventKind {
    /// Task started executing.
    Started,
    /// Progress was updated.
    Progress,
    /// Task completed successfully.
    Completed,
    /// Task failed with an error.
    Failed,
    /// Task was cancelled.
    Cancelled,
}

/// Event emitted during task execution.
#[derive(Debug, Clone)]
pub struct TaskEvent {
    /// Task identifier.
    pub task_id: TaskId,
    /// Stable task name.
    pub name: &'static str,
    /// Event kind.
    pub kind: TaskEventKind,
    /// Progress snapshot when relevant.
    pub progress: Option<TaskProgress>,
    /// Error message when relevant.
    pub error: Option<String>,
}

/// Listener for task lifecycle events.
pub trait TaskEventListener: Send + Sync {
    /// Called when a task event is emitted.
    fn on_event(&self, event: &TaskEvent);
}
