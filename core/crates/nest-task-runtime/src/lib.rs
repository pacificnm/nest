//! Tokio-backed task runtime for the Nest framework.
//!
//! Provides [`TaskRuntime`] (owned or adopted Tokio runtime) and
//! [`TaskManagerService`] for scheduling first-class [`nest_task::Task`] values.

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

mod config;
mod events;
mod manager;
mod module;
pub mod prelude;
mod registry;
mod runtime;

pub use config::{RuntimeConfig, TaskManagerConfig};
pub use manager::TaskManagerService;
pub use module::{TaskRuntimeModule, TASK_RUNTIME_MODULE_ID};
pub use registry::{TaskRecord, TaskRegistry};
pub use runtime::TaskRuntime;

pub use nest_core::{AppBuilder, AppContext, Lifecycle, Module, ModuleId, Service};
pub use nest_error::{NestError, NestResult};
pub use nest_task::{
    CancelToken, Task, TaskContext, TaskError, TaskEvent, TaskEventKind, TaskEventListener,
    TaskHandle, TaskId, TaskManager, TaskProgress, TaskStatus,
};

/// Converts a [`TaskError`] into a [`NestError`].
pub fn task_error_to_nest_error(error: TaskError) -> NestError {
    let mut nest_error = NestError::task(error.message())
        .with_code(error.nest_code())
        .with_module("nest-task-runtime");

    if let Some(task_id) = error.task_id() {
        nest_error = nest_error.with_operation(format!("task_id: {task_id}"));
    }

    nest_error.with_source(error)
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use async_trait::async_trait;
    use nest_core::AppBuilder;
    use nest_error::{codes, NestErrorKind};
    use nest_task::{Task, TaskContext, TaskEventKind, TaskEventListener, TaskManager};

    use super::*;

    struct EchoTask {
        value: u32,
    }

    #[async_trait]
    impl Task for EchoTask {
        type Output = u32;

        fn name(&self) -> &'static str {
            "echo"
        }

        async fn run(&self, ctx: TaskContext) -> NestResult<u32> {
            ctx.progress().set_percent(50.0);
            ctx.progress().set_message("working");
            tokio::time::sleep(Duration::from_millis(5)).await;
            Ok(self.value)
        }
    }

    struct HoldTask {
        release: Arc<tokio::sync::Notify>,
    }

    #[async_trait]
    impl Task for HoldTask {
        type Output = ();

        fn name(&self) -> &'static str {
            "hold"
        }

        async fn run(&self, ctx: TaskContext) -> NestResult<()> {
            tokio::select! {
                _ = self.release.notified() => Ok(()),
                _ = async {
                    while !ctx.cancel_token().is_cancelled() {
                        tokio::time::sleep(Duration::from_millis(5)).await;
                    }
                } => Err(TaskError::cancelled("hold task cancelled").into()),
            }
        }
    }

    struct SlowTask;

    #[async_trait]
    impl Task for SlowTask {
        type Output = ();

        fn name(&self) -> &'static str {
            "slow"
        }

        async fn run(&self, ctx: TaskContext) -> NestResult<()> {
            for _ in 0..20 {
                if ctx.cancel_token().is_cancelled() {
                    return Err(TaskError::cancelled("slow task cancelled").into());
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            Ok(())
        }
    }

    struct RecordingListener {
        events: Arc<Mutex<Vec<TaskEventKind>>>,
    }

    impl TaskEventListener for RecordingListener {
        fn on_event(&self, event: &TaskEvent) {
            self.events.lock().unwrap().push(event.kind);
        }
    }

    async fn manager_from_current() -> (nest_core::BuiltApp, TaskManagerService) {
        let mut built = AppBuilder::new()
            .module(
                TaskRuntimeModule::from_current()
                    .unwrap()
                    .with_manager_config(TaskManagerConfig {
                        max_concurrent: 4,
                    }),
            )
            .build()
            .unwrap();
        built.startup().unwrap();
        let manager = built.context.service::<TaskManagerService>().unwrap().clone();
        (built, manager)
    }

    #[tokio::test]
    async fn spawn_and_await_result() {
        let (_built, manager) = manager_from_current().await;
        let handle = manager.spawn(EchoTask { value: 9 }).await.unwrap();
        assert_eq!(handle.await_result().await.unwrap(), 9);
        assert_eq!(handle.status(), TaskStatus::Completed);
    }

    #[tokio::test]
    async fn progress_updates_visible_on_handle() {
        let (_built, manager) = manager_from_current().await;
        let handle = manager.spawn(EchoTask { value: 1 }).await.unwrap();
        let _ = handle.await_result().await.unwrap();
        assert_eq!(handle.progress().percent, Some(50.0));
        assert_eq!(handle.progress().message.as_deref(), Some("working"));
    }

    #[tokio::test]
    async fn cancel_marks_task_cancelled() {
        let (_built, manager) = manager_from_current().await;
        let handle = manager.spawn(SlowTask).await.unwrap();
        handle.cancel();
        let result = handle.await_result().await;
        assert!(result.is_err());
        assert_eq!(handle.status(), TaskStatus::Cancelled);
    }

    #[tokio::test]
    async fn concurrency_limit_serializes_tasks() {
        let release = Arc::new(tokio::sync::Notify::new());
        let mut built = AppBuilder::new()
            .module(
                TaskRuntimeModule::from_current()
                    .unwrap()
                    .with_manager_config(TaskManagerConfig { max_concurrent: 1 }),
            )
            .build()
            .unwrap();
        built.startup().unwrap();
        let manager = built.context.service::<TaskManagerService>().unwrap().clone();

        let first = manager
            .spawn(HoldTask {
                release: Arc::clone(&release),
            })
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_millis(20)).await;
        let second = manager
            .spawn(HoldTask {
                release: Arc::clone(&release),
            })
            .await
            .unwrap();
        assert_eq!(first.status(), TaskStatus::Running);
        assert_eq!(second.status(), TaskStatus::Queued);

        first.cancel();
        let _ = first.await_result().await;
        tokio::time::sleep(Duration::from_millis(20)).await;
        assert_eq!(second.status(), TaskStatus::Running);
        second.cancel();
        let _ = second.await_result().await;
    }

    #[tokio::test]
    async fn spawn_blocking_runs_off_async_executor() {
        let (_built, manager) = manager_from_current().await;
        let handle = manager
            .spawn_blocking("blocking", || Ok::<_, NestError>(99))
            .await
            .unwrap();
        assert_eq!(handle.await_result().await.unwrap(), 99);
    }

    #[tokio::test]
    async fn task_events_recorded_by_listener() {
        let (_built, manager) = manager_from_current().await;
        let kinds = Arc::new(Mutex::new(Vec::new()));
        manager.add_listener(Arc::new(RecordingListener {
            events: Arc::clone(&kinds),
        }));

        let handle = manager.spawn(EchoTask { value: 3 }).await.unwrap();
        let _ = handle.await_result().await.unwrap();

        let recorded = kinds.lock().unwrap().clone();
        assert!(recorded.contains(&TaskEventKind::Started));
        assert!(recorded.contains(&TaskEventKind::Completed));
    }

    #[tokio::test]
    async fn cancelled_task_maps_to_nest_error_code() {
        let (_built, manager) = manager_from_current().await;
        let handle = manager.spawn(SlowTask).await.unwrap();
        handle.cancel();
        let err = handle.await_result().await.unwrap_err();
        assert_eq!(err.kind(), NestErrorKind::Task);
        assert_eq!(err.code(), Some(codes::NEST_TASK_CANCELLED));
    }

    #[test]
    fn owned_runtime_executes_task() {
        let runtime = TaskRuntime::new_owned(RuntimeConfig {
            worker_threads: 2,
            thread_name_prefix: "nest-task-test".to_string(),
        })
        .unwrap();

        let mut built = AppBuilder::new()
            .module(TaskRuntimeModule::new(runtime.clone(), TaskManagerConfig::default()))
            .build()
            .unwrap();
        built.startup().unwrap();

        runtime.handle().block_on(async {
            let manager = built.context.service::<TaskManagerService>().unwrap();
            let handle = manager.spawn(EchoTask { value: 5 }).await.unwrap();
            assert_eq!(handle.await_result().await.unwrap(), 5);
        });
    }

    #[test]
    fn task_error_converts_to_nest_error() {
        let error = TaskError::cancelled("done");
        let nest_error = task_error_to_nest_error(error);
        assert_eq!(nest_error.kind(), NestErrorKind::Task);
        assert_eq!(nest_error.code(), Some(codes::NEST_TASK_CANCELLED));
    }
}
