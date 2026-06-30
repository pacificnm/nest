//! Task progress reporting.

use std::sync::Arc;

/// Progress snapshot for a task.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TaskProgress {
    /// Completion percentage from 0.0 to 100.0.
    pub percent: Option<f32>,
    /// Human-readable progress message.
    pub message: Option<String>,
    /// Current step or item count.
    pub current: Option<u64>,
    /// Total steps or items, when known.
    pub total: Option<u64>,
}

impl TaskProgress {
    /// Clamps percent into the 0–100 range.
    pub fn clamped_percent(&self) -> Option<f32> {
        self.percent.map(|value| value.clamp(0.0, 100.0))
    }
}

/// Reports progress updates for the current task.
#[derive(Clone)]
pub struct ProgressReporter {
    sink: Arc<dyn Fn(TaskProgress) + Send + Sync>,
    last: Arc<std::sync::Mutex<TaskProgress>>,
}

impl ProgressReporter {
    /// Creates a reporter that forwards updates to the given sink.
    pub fn new(sink: Arc<dyn Fn(TaskProgress) + Send + Sync>) -> Self {
        Self {
            sink,
            last: Arc::new(std::sync::Mutex::new(TaskProgress::default())),
        }
    }

    /// Creates a no-op reporter.
    pub fn noop() -> Self {
        Self::new(Arc::new(|_| {}))
    }

    /// Sets the completion percentage.
    pub fn set_percent(&self, percent: f32) {
        let mut progress = self.last.lock().unwrap();
        progress.percent = Some(percent.clamp(0.0, 100.0));
        (self.sink)(progress.clone());
    }

    /// Sets the progress message.
    pub fn set_message(&self, message: impl Into<String>) {
        let mut progress = self.last.lock().unwrap();
        progress.message = Some(message.into());
        (self.sink)(progress.clone());
    }

    /// Sets current and total counts.
    pub fn set_current(&self, current: u64, total: u64) {
        let mut progress = self.last.lock().unwrap();
        progress.current = Some(current);
        progress.total = Some(total);
        progress.percent = if total == 0 {
            None
        } else {
            Some((current as f32 / total as f32) * 100.0)
        };
        (self.sink)(progress.clone());
    }

    /// Applies a full progress snapshot.
    pub fn update(&self, progress: TaskProgress) {
        *self.last.lock().unwrap() = progress.clone();
        (self.sink)(progress);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[test]
    fn clamps_percent() {
        let progress = TaskProgress {
            percent: Some(150.0),
            ..TaskProgress::default()
        };
        assert_eq!(progress.clamped_percent(), Some(100.0));
    }

    #[test]
    fn reporter_updates_sink() {
        let last = Arc::new(Mutex::new(TaskProgress::default()));
        let capture = Arc::clone(&last);
        let reporter = ProgressReporter::new(Arc::new(move |progress| {
            *capture.lock().unwrap() = progress;
        }));

        reporter.set_message("working");
        assert_eq!(last.lock().unwrap().message.as_deref(), Some("working"));
    }
}
