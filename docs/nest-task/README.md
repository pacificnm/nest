# nest-task

Task execution contracts for the [Nest framework](../../README.md).

**Crate path:** [`crates/nest-task`](../../crates/nest-task)

## Role

nest-task defines first-class background work: tasks, handles, progress, cancellation, and events. It does **not** own a runtime — see [`nest-task-runtime`](../nest-task-runtime/README.md).

| Type | Purpose |
|------|---------|
| `Task` | Async work unit with `name()` and `run(ctx)` |
| `TaskContext` | Task id, `AppContext`, cancel token, progress, tracing span |
| `TaskHandle<O>` | `id()`, `status()`, `progress()`, `cancel()`, `await_result()` |
| `TaskManager` | `spawn()` and `spawn_blocking()` trait |
| `TaskStatus` | Queued, Running, Completed, Failed, Cancelled |
| `TaskProgress` | Percent, message, current/total counts |
| `CancelToken` | Cooperative cancellation |
| `TaskEvent` | Started, Progress, Completed, Failed, Cancelled |

## Task trait

```rust
#[async_trait]
pub trait Task: Send + Sync + 'static {
    type Output: Send + Sync + 'static;
    fn name(&self) -> &'static str;
    async fn run(&self, ctx: TaskContext) -> NestResult<Self::Output>;
}
```

## Progress

```rust
ctx.progress().set_percent(63.0);
ctx.progress().set_message("Indexing repository...");
ctx.progress().set_current(80, 100);
```

## Cancellation

```rust
if ctx.cancel_token().is_cancelled() {
    return Err(TaskError::cancelled("stopped").into());
}
```

## nest-core `Job` trait

`nest-core::Job` remains metadata-only for introspection. Use `Task::name()` as the runtime identifier; types may implement both where `Job::id()` matches `name()`.

## Related

- [nest-task-runtime](../nest-task-runtime/README.md) — Tokio executor and module
