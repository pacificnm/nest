//! Task execution contracts for the Nest framework.
//!
//! nest-task defines first-class tasks, handles, progress, cancellation, and
//! events. It does not own a runtime — see `nest-task-runtime`.

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

pub mod cancel;
pub mod codes;
pub mod context;
pub mod error;
pub mod events;
pub mod handle;
pub mod id;
pub mod manager;
pub mod prelude;
pub mod progress;
pub mod status;
pub mod task;

pub use cancel::CancelToken;
pub use context::TaskContext;
pub use error::{TaskError, TaskErrorKind, TaskResult};
pub use events::{TaskEvent, TaskEventKind, TaskEventListener};
pub use handle::{TaskHandle, TaskHandleBackend};
pub use id::TaskId;
pub use manager::TaskManager;
pub use progress::{ProgressReporter, TaskProgress};
pub use status::TaskStatus;
pub use task::Task;
