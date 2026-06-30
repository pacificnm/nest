//! Task identifiers.

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

static TASK_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Unique identifier for a running or completed task.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TaskId(pub String);

impl TaskId {
    /// Creates a new unique task id.
    pub fn new() -> Self {
        let id = TASK_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self(format!("task-{id}"))
    }

    /// Creates a task id from an existing string.
    pub fn from_string(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the id string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}
