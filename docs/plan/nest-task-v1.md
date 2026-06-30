# nest-task v1 Implementation Plan

## Status: Implemented

See [nest-task docs](../nest-task/README.md) and [nest-task-runtime docs](../nest-task-runtime/README.md).

## Context

nest-task is the execution engine for asynchronous, long-running work. Domain crates define *what* the work is; nest-task defines *how* it runs — scheduling, progress, cancellation, and events.

## Crate boundaries

| Crate | Role |
|-------|------|
| `nest-task` | `Task`, `TaskContext`, `TaskHandle`, progress, cancellation, events (no Tokio) |
| `nest-task-runtime` | `TaskRuntime`, `TaskManagerService`, `TaskRuntimeModule` (Tokio) |
| `nest-core` | Sync module registry; lifecycle passes `Arc<AppContext>` |

## Runtime ownership

| Host | Pattern |
|------|---------|
| nest-cli / nest-app / Kiwi | `TaskRuntimeModule::owned(RuntimeConfig::default())` |
| nest-serve / axum / tests | `TaskRuntimeModule::from_current()` |

## v1 limitations

- No cron, delayed scheduling, or task graphs
- No persistent task history
- No `nest-task-ui`, `nest-task-cli`, or `nest-task-web`
- nest-http-client unchanged — wrap long HTTP work in `TaskManager::spawn`

## Follow-up

- Task graphs and queues
- Platform progress adapters
- Inject `TaskRuntime` handle into nest-http-client v2
