//! Nest module and lifecycle integration.

use std::sync::Arc;

use nest_core::{AppBuilder, Lifecycle, Module, ModuleId, NestResult};

use crate::config::{RuntimeConfig, TaskManagerConfig};
use crate::manager::TaskManagerService;
use crate::runtime::TaskRuntime;

/// Module id for [`TaskRuntimeModule`].
pub const TASK_RUNTIME_MODULE_ID: ModuleId = ModuleId("nest-task-runtime");

/// Registers [`TaskRuntime`] and [`TaskManagerService`].
pub struct TaskRuntimeModule {
    runtime: TaskRuntime,
    manager_config: TaskManagerConfig,
}

impl TaskRuntimeModule {
    /// Creates a module from an existing runtime and manager config.
    pub fn new(runtime: TaskRuntime, manager_config: TaskManagerConfig) -> Self {
        Self {
            runtime,
            manager_config,
        }
    }

    /// Creates a module with an owned Tokio runtime.
    pub fn owned(runtime_config: RuntimeConfig) -> NestResult<Self> {
        Ok(Self::new(
            TaskRuntime::new_owned(runtime_config)?,
            TaskManagerConfig::default(),
        ))
    }

    /// Creates a module that adopts the current Tokio runtime handle.
    pub fn from_current() -> NestResult<Self> {
        Ok(Self::new(
            TaskRuntime::from_current()?,
            TaskManagerConfig::default(),
        ))
    }

    /// Creates a module that adopts an existing runtime handle.
    pub fn from_handle(handle: tokio::runtime::Handle) -> Self {
        Self::new(
            TaskRuntime::from_handle(handle),
            TaskManagerConfig::default(),
        )
    }

    /// Sets task manager configuration.
    pub fn with_manager_config(mut self, config: TaskManagerConfig) -> Self {
        self.manager_config = config;
        self
    }
}

struct TaskManagerLifecycle {
    manager: TaskManagerService,
}

impl Lifecycle for TaskManagerLifecycle {
    fn on_startup(&mut self, ctx: Arc<nest_core::AppContext>) -> NestResult<()> {
        self.manager.set_context(ctx);
        Ok(())
    }

    fn on_shutdown(&mut self, _ctx: Arc<nest_core::AppContext>) -> NestResult<()> {
        self.manager.cancel_all();
        Ok(())
    }
}

impl Module for TaskRuntimeModule {
    fn id(&self) -> ModuleId {
        TASK_RUNTIME_MODULE_ID
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        let manager =
            TaskManagerService::new(self.runtime.handle().clone(), self.manager_config.clone());
        app.register_service(self.runtime.clone())?;
        app.register_service(manager.clone())?;
        app.register_lifecycle(TaskManagerLifecycle { manager });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nest_core::AppBuilder;
    use nest_task::TaskManager;

    #[test]
    fn module_registers_services() {
        let runtime = TaskRuntime::new_owned(RuntimeConfig::default()).unwrap();
        let built = AppBuilder::new()
            .module(TaskRuntimeModule::new(
                runtime,
                TaskManagerConfig::default(),
            ))
            .build()
            .unwrap();

        assert!(built.context.has_service::<TaskRuntime>());
        assert!(built.context.has_service::<TaskManagerService>());
    }

    #[tokio::test]
    async fn lifecycle_attaches_context() {
        let built = AppBuilder::new()
            .module(TaskRuntimeModule::from_current().unwrap())
            .build()
            .unwrap();
        let mut built = built;
        built.startup().unwrap();

        let manager = built.context.service::<TaskManagerService>().unwrap();
        let task = EchoTask { value: 7 };
        let handle = manager.spawn(task).await.unwrap();
        assert_eq!(handle.await_result().await.unwrap(), 7);
    }

    use async_trait::async_trait;
    use nest_task::{Task, TaskContext};

    struct EchoTask {
        value: u32,
    }

    #[async_trait]
    impl Task for EchoTask {
        type Output = u32;

        fn name(&self) -> &'static str {
            "echo"
        }

        async fn run(&self, _ctx: TaskContext) -> NestResult<u32> {
            Ok(self.value)
        }
    }
}
