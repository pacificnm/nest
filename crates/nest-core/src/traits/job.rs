//! Job trait stub for future nest-tasks integration.

/// Marker trait for background jobs. Execution is provided by `nest-task-runtime`.
pub trait Job: Send + Sync + 'static {
    /// Returns a stable identifier for this job type.
    fn id(&self) -> &str;
}
