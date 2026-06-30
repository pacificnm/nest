# nest-task-runtime

Tokio-backed task runtime for the Nest framework.

**Crate path:** [`core/crates/nest-task-runtime`](../../core/crates/nest-task-runtime)

## Quick start

### Nest-owned runtime (CLI / desktop)

```rust
use nest_core::AppBuilder;
use nest_task_runtime::{RuntimeConfig, TaskManagerService, TaskRuntimeModule};
use nest_task::TaskManager;

let mut built = AppBuilder::new()
    .module(TaskRuntimeModule::owned(RuntimeConfig::default())?)
    .build()?;
built.startup()?;

// Run async work on the runtime handle:
let runtime = built.context.service::<nest_task_runtime::TaskRuntime>()?;
runtime.handle().block_on(async {
    let tasks = built.context.service::<TaskManagerService>()?;
    let handle = tasks.spawn(ImportCsvTask { path }).await?;
    handle.await_result().await
})?;
```

### Adopted runtime (server / tests)

```rust
#[tokio::main]
async fn main() -> nest_error::NestResult<()> {
    let mut built = AppBuilder::new()
        .module(TaskRuntimeModule::from_current()?)
        .build()?;
    built.startup()?;

    let tasks = built.context.service::<TaskManagerService>()?;
    let handle = tasks.spawn(CloneRepositoryTask { url, path }).await?;
    handle.await_result().await
}
```

## Runtime ownership rule

| Situation | Use |
|-----------|-----|
| Nest starts the process | `TaskRuntimeModule::owned(...)` |
| Another async host owns Tokio | `TaskRuntimeModule::from_current()` |

## TaskManagerService

| Feature | Detail |
|---------|--------|
| `spawn` | Async task with concurrency limit |
| `spawn_blocking` | CPU/blocking work on Tokio blocking pool |
| Registry | In-memory task status and progress |
| Events | `tracing` + optional `TaskEventListener` |
| Shutdown | Lifecycle cancels active tasks |

Concurrency slots are acquired when a task **starts**, not when `spawn().await` returns — queued tasks get `TaskStatus::Queued` immediately.

## Configuration

```rust
TaskRuntimeModule::owned(RuntimeConfig::default())?
    .with_manager_config(TaskManagerConfig { max_concurrent: 8 })
```

## Module

```rust
pub const TASK_RUNTIME_MODULE_ID: ModuleId = ModuleId("nest-task-runtime");
```

Registers `TaskRuntime`, `TaskManagerService`, and a lifecycle handler that attaches `AppContext` on startup.

## Related

- [nest-task](../nest-task/README.md) — contracts
- [nest-http-client](../nest-http-client/README.md) — short HTTP calls stay direct; long work uses `TaskManager::spawn`
