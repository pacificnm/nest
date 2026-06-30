//! Runtime and task manager configuration.

/// Configuration for an owned Tokio runtime.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Number of worker threads for the multi-thread runtime.
    pub worker_threads: usize,
    /// Thread name prefix for worker threads.
    pub thread_name_prefix: String,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            worker_threads: 4,
            thread_name_prefix: "nest-task".to_string(),
        }
    }
}

/// Configuration for the task manager.
#[derive(Debug, Clone)]
pub struct TaskManagerConfig {
    /// Maximum number of tasks executing concurrently.
    pub max_concurrent: usize,
}

impl Default for TaskManagerConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 4,
        }
    }
}
