//! In-memory task registry.

use std::collections::HashMap;
use std::sync::RwLock;

use nest_task::{CancelToken, TaskId, TaskProgress, TaskStatus};

/// Snapshot of a registered task.
#[derive(Debug, Clone)]
pub struct TaskRecord {
    /// Stable task name.
    pub name: &'static str,
    /// Current status.
    pub status: TaskStatus,
    /// Latest progress snapshot.
    pub progress: TaskProgress,
    /// Shared cancellation token.
    pub cancel: CancelToken,
}

/// In-memory registry of active and recently finished tasks.
#[derive(Default)]
pub struct TaskRegistry {
    tasks: RwLock<HashMap<TaskId, TaskRecord>>,
}

impl TaskRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts or replaces a task record.
    pub fn insert(&self, id: TaskId, record: TaskRecord) {
        self.tasks.write().unwrap().insert(id, record);
    }

    /// Updates the status for a task.
    pub fn set_status(&self, id: &TaskId, status: TaskStatus) {
        if let Some(record) = self.tasks.write().unwrap().get_mut(id) {
            record.status = status;
        }
    }

    /// Updates progress for a task.
    pub fn set_progress(&self, id: &TaskId, progress: TaskProgress) {
        if let Some(record) = self.tasks.write().unwrap().get_mut(id) {
            record.progress = progress.clone();
        }
    }

    /// Returns a task record clone, if present.
    pub fn get(&self, id: &TaskId) -> Option<TaskRecord> {
        self.tasks.read().unwrap().get(id).cloned()
    }

    /// Returns all task ids currently tracked.
    pub fn ids(&self) -> Vec<TaskId> {
        self.tasks.read().unwrap().keys().cloned().collect()
    }
}
