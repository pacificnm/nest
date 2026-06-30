//! Task event emission.

use nest_task::{TaskEvent, TaskEventKind, TaskEventListener};
use tracing::{debug, error, info, warn};

/// Emits structured tracing and forwards to listeners.
pub fn emit_event(event: &TaskEvent, listeners: &[std::sync::Arc<dyn TaskEventListener>]) {
    match event.kind {
        TaskEventKind::Started => {
            info!(
                task.id = %event.task_id,
                task.name = event.name,
                "task started"
            );
        }
        TaskEventKind::Progress => {
            debug!(
                task.id = %event.task_id,
                task.name = event.name,
                progress.percent = ?event.progress.as_ref().and_then(|p| p.percent),
                progress.message = ?event.progress.as_ref().and_then(|p| p.message.as_deref()),
                "task progress"
            );
        }
        TaskEventKind::Completed => {
            info!(
                task.id = %event.task_id,
                task.name = event.name,
                "task completed"
            );
        }
        TaskEventKind::Failed => {
            error!(
                task.id = %event.task_id,
                task.name = event.name,
                error = ?event.error,
                "task failed"
            );
        }
        TaskEventKind::Cancelled => {
            warn!(
                task.id = %event.task_id,
                task.name = event.name,
                "task cancelled"
            );
        }
    }

    for listener in listeners {
        listener.on_event(event);
    }
}
