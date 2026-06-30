//! Task errors.

use std::error::Error;
use std::fmt;

use crate::codes::{
    NEST_TASK_CANCELLED, NEST_TASK_NOT_FOUND, NEST_TASK_RUNTIME_MISSING, NEST_TASK_SPAWN_FAILED,
};
use crate::id::TaskId;

/// Result type for task operations.
pub type TaskResult<T> = Result<T, TaskError>;

/// High-level category for a task error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TaskErrorKind {
    /// Task spawn failed.
    Spawn,
    /// Task was cancelled.
    Cancelled,
    /// Task was not found.
    NotFound,
    /// Runtime is not available.
    Runtime,
    /// Task execution failed.
    Execution,
    /// Configuration error.
    Config,
}

/// Structured error for nest-task and task adapters.
#[derive(Debug)]
pub struct TaskError {
    kind: TaskErrorKind,
    message: String,
    code: Option<String>,
    task_id: Option<TaskId>,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl TaskError {
    /// Creates a new task error.
    pub fn new(kind: TaskErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            code: None,
            task_id: None,
            source: None,
        }
    }

    /// Creates a spawn error.
    pub fn spawn(message: impl Into<String>) -> Self {
        Self::new(TaskErrorKind::Spawn, message).with_code(NEST_TASK_SPAWN_FAILED)
    }

    /// Creates a cancelled error.
    pub fn cancelled(message: impl Into<String>) -> Self {
        Self::new(TaskErrorKind::Cancelled, message).with_code(NEST_TASK_CANCELLED)
    }

    /// Creates a not-found error.
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(TaskErrorKind::NotFound, message).with_code(NEST_TASK_NOT_FOUND)
    }

    /// Creates a runtime-missing error.
    pub fn runtime_missing(message: impl Into<String>) -> Self {
        Self::new(TaskErrorKind::Runtime, message).with_code(NEST_TASK_RUNTIME_MISSING)
    }

    /// Creates an execution error.
    pub fn execution(message: impl Into<String>) -> Self {
        Self::new(TaskErrorKind::Execution, message)
    }

    /// Creates a config error.
    pub fn config(message: impl Into<String>) -> Self {
        Self::new(TaskErrorKind::Config, message)
    }

    /// Sets a stable error code.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Sets the task id context.
    pub fn with_task_id(mut self, task_id: TaskId) -> Self {
        self.task_id = Some(task_id);
        self
    }

    /// Attaches a source error.
    pub fn with_source(mut self, source: impl Error + Send + Sync + 'static) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    /// Returns the error kind.
    pub fn kind(&self) -> TaskErrorKind {
        self.kind
    }

    /// Returns the message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the stable code, if set.
    pub fn code(&self) -> Option<&str> {
        self.code.as_deref()
    }

    /// Returns the task id context, if set.
    pub fn task_id(&self) -> Option<&TaskId> {
        self.task_id.as_ref()
    }

    /// Default code when converting to [`nest_error::NestError`].
    pub fn nest_code(&self) -> &str {
        self.code.as_deref().unwrap_or(NEST_TASK_SPAWN_FAILED)
    }
}

impl fmt::Display for TaskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for TaskError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|source| source.as_ref() as &(dyn Error + 'static))
    }
}

impl From<TaskError> for nest_error::NestError {
    fn from(error: TaskError) -> nest_error::NestError {
        let mut nest_error = nest_error::NestError::task(error.message())
            .with_code(error.nest_code())
            .with_module("nest-task");

        if let Some(task_id) = error.task_id() {
            nest_error = nest_error.with_operation(format!("task_id: {task_id}"));
        }

        nest_error.with_source(error)
    }
}
