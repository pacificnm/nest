//! Tokio-backed task manager.

use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, RwLock};

use async_trait::async_trait;
use nest_core::AppContext;
use nest_error::NestResult;
use nest_task::{
    CancelToken, Task, TaskContext, TaskError, TaskEvent, TaskEventKind, TaskEventListener,
    TaskHandle, TaskHandleBackend, TaskId, TaskManager, TaskProgress, TaskStatus,
};
use tokio::runtime::Handle;
use tokio::sync::{oneshot, watch, Semaphore};
use tokio::task::AbortHandle;
use tracing::info_span;

use crate::config::TaskManagerConfig;
use crate::events::emit_event;
use crate::registry::{TaskRecord, TaskRegistry};

struct TaskManagerInner {
    runtime: Handle,
    app: RwLock<Option<Arc<AppContext>>>,
    #[allow(dead_code)]
    config: TaskManagerConfig,
    registry: Arc<TaskRegistry>,
    semaphore: Arc<Semaphore>,
    listeners: Arc<RwLock<Vec<Arc<dyn TaskEventListener>>>>,
}

/// Schedules, tracks, and executes background tasks.
#[derive(Clone)]
pub struct TaskManagerService {
    inner: Arc<TaskManagerInner>,
}

struct SharedTaskState<O> {
    id: TaskId,
    name: &'static str,
    cancel: CancelToken,
    status: watch::Sender<TaskStatus>,
    status_rx: watch::Receiver<TaskStatus>,
    #[allow(dead_code)]
    progress: watch::Sender<TaskProgress>,
    progress_rx: watch::Receiver<TaskProgress>,
    result_tx: Mutex<Option<oneshot::Sender<NestResult<O>>>>,
    abort: Mutex<Option<AbortHandle>>,
    registry: Arc<TaskRegistry>,
    listeners: Arc<RwLock<Vec<Arc<dyn TaskEventListener>>>>,
}

struct TokioTaskHandle<O> {
    state: Arc<SharedTaskState<O>>,
    result_rx: Mutex<Option<oneshot::Receiver<NestResult<O>>>>,
}

impl<O> TaskHandleBackend<O> for TokioTaskHandle<O>
where
    O: Send + Sync + 'static,
{
    fn id(&self) -> TaskId {
        self.state.id.clone()
    }

    fn status(&self) -> TaskStatus {
        *self.state.status_rx.borrow()
    }

    fn progress(&self) -> TaskProgress {
        self.state.progress_rx.borrow().clone()
    }

    fn cancel(&self) {
        if self.status().is_finished() {
            return;
        }

        self.state.cancel.cancel();
        if let Some(abort) = self.state.abort.lock().unwrap().take() {
            abort.abort();
        }
        let _ = self.state.status.send(TaskStatus::Cancelled);
        self.state
            .registry
            .set_status(&self.state.id, TaskStatus::Cancelled);

        emit_event(
            &TaskEvent {
                task_id: self.state.id.clone(),
                name: self.state.name,
                kind: TaskEventKind::Cancelled,
                progress: Some(self.progress()),
                error: None,
            },
            &self.state.listeners.read().unwrap(),
        );

        if let Some(tx) = self.state.result_tx.lock().unwrap().take() {
            let _ = tx.send(Err(TaskError::cancelled(format!(
                "task `{}` cancelled",
                self.state.name
            ))
            .into()));
        }
    }

    fn is_finished(&self) -> bool {
        self.status().is_finished()
    }

    fn await_result(&self) -> Pin<Box<dyn Future<Output = NestResult<O>> + Send + '_>> {
        let receiver = self.result_rx.lock().unwrap().take();
        Box::pin(async move {
            match receiver {
                Some(rx) => rx.await.unwrap_or_else(|_| {
                    Err(TaskError::execution("task result channel closed").into())
                }),
                None => Err(TaskError::execution("task result already consumed").into()),
            }
        })
    }
}

impl TaskManagerService {
    /// Creates a task manager bound to the given runtime handle.
    pub fn new(runtime: Handle, config: TaskManagerConfig) -> Self {
        let max_concurrent = config.max_concurrent.max(1);
        Self {
            inner: Arc::new(TaskManagerInner {
                runtime,
                app: RwLock::new(None),
                config: TaskManagerConfig { max_concurrent },
                registry: Arc::new(TaskRegistry::new()),
                semaphore: Arc::new(Semaphore::new(max_concurrent)),
                listeners: Arc::new(RwLock::new(Vec::new())),
            }),
        }
    }

    /// Attaches the application context for task execution.
    pub fn set_context(&self, ctx: Arc<AppContext>) {
        *self.inner.app.write().unwrap() = Some(ctx);
    }

    /// Registers a task event listener.
    pub fn add_listener(&self, listener: Arc<dyn TaskEventListener>) {
        self.inner.listeners.write().unwrap().push(listener);
    }

    /// Returns the in-memory task registry.
    pub fn registry(&self) -> &TaskRegistry {
        &self.inner.registry
    }

    /// Requests cancellation for all non-finished tasks.
    pub fn cancel_all(&self) {
        for id in self.inner.registry.ids() {
            if let Some(record) = self.inner.registry.get(&id) {
                if !record.status.is_finished() {
                    record.cancel.cancel();
                }
            }
        }
    }

    fn app_context(&self) -> NestResult<Arc<AppContext>> {
        self.inner
            .app
            .read()
            .unwrap()
            .clone()
            .ok_or_else(|| TaskError::config("application context not attached").into())
    }

    fn emit(&self, event: TaskEvent) {
        emit_event(&event, &self.inner.listeners.read().unwrap());
    }

    async fn prepare_spawn(&self, name: &'static str) -> NestResult<(TaskId, CancelToken)> {
        let _ = self.app_context()?;

        let id = TaskId::new();
        let cancel = CancelToken::new();
        self.inner.registry.insert(
            id.clone(),
            TaskRecord {
                name,
                status: TaskStatus::Queued,
                progress: TaskProgress::default(),
                cancel: cancel.clone(),
            },
        );
        Ok((id, cancel))
    }

    fn build_handle<O: Send + Sync + 'static>(
        state: Arc<SharedTaskState<O>>,
        result_rx: oneshot::Receiver<NestResult<O>>,
    ) -> TaskHandle<O> {
        TaskHandle::new(Arc::new(TokioTaskHandle {
            state,
            result_rx: Mutex::new(Some(result_rx)),
        }))
    }
}

#[async_trait]
impl TaskManager for TaskManagerService {
    async fn spawn<T>(&self, task: T) -> NestResult<TaskHandle<T::Output>>
    where
        T: Task,
    {
        let name = task.name();
        let (id, cancel) = self.prepare_spawn(name).await?;
        let app = self.app_context()?;

        let (status_tx, status_rx) = watch::channel(TaskStatus::Queued);
        let (progress_tx, progress_rx) = watch::channel(TaskProgress::default());
        let (result_tx, result_rx) = oneshot::channel();

        let state = Arc::new(SharedTaskState {
            id: id.clone(),
            name,
            cancel: cancel.clone(),
            status: status_tx.clone(),
            status_rx,
            progress: progress_tx.clone(),
            progress_rx,
            result_tx: Mutex::new(Some(result_tx)),
            abort: Mutex::new(None),
            registry: Arc::clone(&self.inner.registry),
            listeners: Arc::clone(&self.inner.listeners),
        });

        let handle = Self::build_handle(Arc::clone(&state), result_rx);
        let manager = self.clone();
        let span = info_span!("task", task.id = %id, task.name = name);
        let state_for_abort = Arc::clone(&state);
        let semaphore = Arc::clone(&self.inner.semaphore);

        let join = self.inner.runtime.spawn(async move {
            let permit = match semaphore.acquire_owned().await {
                Ok(permit) => permit,
                Err(_) => {
                    finish_cancelled(&manager, &state, &id, name);
                    return;
                }
            };
            let _permit = permit;
            let _span_guard = span.enter();

            let _ = status_tx.send(TaskStatus::Running);
            manager.inner.registry.set_status(&id, TaskStatus::Running);
            manager.emit(TaskEvent {
                task_id: id.clone(),
                name,
                kind: TaskEventKind::Started,
                progress: None,
                error: None,
            });

            if cancel.is_cancelled() {
                finish_cancelled(&manager, &state, &id, name);
                return;
            }

            let progress_sink = {
                let progress_tx = progress_tx.clone();
                let registry = Arc::clone(&manager.inner.registry);
                let listeners = Arc::clone(&manager.inner.listeners);
                let task_id = id.clone();
                Arc::new(move |progress: TaskProgress| {
                    let _ = progress_tx.send(progress.clone());
                    registry.set_progress(&task_id, progress.clone());
                    emit_event(
                        &TaskEvent {
                            task_id: task_id.clone(),
                            name,
                            kind: TaskEventKind::Progress,
                            progress: Some(progress),
                            error: None,
                        },
                        &listeners.read().unwrap(),
                    );
                })
            };

            let ctx = TaskContext::new(
                id.clone(),
                app,
                cancel.clone(),
                nest_task::ProgressReporter::new(progress_sink),
                span.clone(),
            );

            match task.run(ctx).await {
                Ok(output) => {
                    let _ = status_tx.send(TaskStatus::Completed);
                    manager
                        .inner
                        .registry
                        .set_status(&id, TaskStatus::Completed);
                    manager.emit(TaskEvent {
                        task_id: id.clone(),
                        name,
                        kind: TaskEventKind::Completed,
                        progress: Some(state.progress_rx.borrow().clone()),
                        error: None,
                    });
                    if let Some(tx) = state.result_tx.lock().unwrap().take() {
                        let _ = tx.send(Ok(output));
                    }
                }
                Err(error) => {
                    let message = error.to_string();
                    let _ = status_tx.send(TaskStatus::Failed);
                    manager.inner.registry.set_status(&id, TaskStatus::Failed);
                    manager.emit(TaskEvent {
                        task_id: id.clone(),
                        name,
                        kind: TaskEventKind::Failed,
                        progress: Some(state.progress_rx.borrow().clone()),
                        error: Some(message.clone()),
                    });
                    if let Some(tx) = state.result_tx.lock().unwrap().take() {
                        let _ = tx.send(Err(error));
                    }
                }
            }
        });

        *state_for_abort.abort.lock().unwrap() = Some(join.abort_handle());

        Ok(handle)
    }

    async fn spawn_blocking<F, R>(&self, name: &'static str, f: F) -> NestResult<TaskHandle<R>>
    where
        F: FnOnce() -> NestResult<R> + Send + 'static,
        R: Send + Sync + 'static,
    {
        let (id, cancel) = self.prepare_spawn(name).await?;
        let app = self.app_context()?;

        let (status_tx, status_rx) = watch::channel(TaskStatus::Queued);
        let (progress_tx, progress_rx) = watch::channel(TaskProgress::default());
        let (result_tx, result_rx) = oneshot::channel();

        let state = Arc::new(SharedTaskState {
            id: id.clone(),
            name,
            cancel: cancel.clone(),
            status: status_tx.clone(),
            status_rx,
            progress: progress_tx.clone(),
            progress_rx,
            result_tx: Mutex::new(Some(result_tx)),
            abort: Mutex::new(None),
            registry: Arc::clone(&self.inner.registry),
            listeners: Arc::clone(&self.inner.listeners),
        });

        let handle = Self::build_handle(Arc::clone(&state), result_rx);
        let manager = self.clone();
        let span = info_span!("task.blocking", task.id = %id, task.name = name);
        let state_for_abort = Arc::clone(&state);
        let semaphore = Arc::clone(&self.inner.semaphore);

        let join = self.inner.runtime.spawn(async move {
            let permit = match semaphore.acquire_owned().await {
                Ok(permit) => permit,
                Err(_) => {
                    finish_cancelled(&manager, &state, &id, name);
                    return;
                }
            };
            let _permit = permit;
            let _span_guard = span.enter();

            let _ = status_tx.send(TaskStatus::Running);
            manager.inner.registry.set_status(&id, TaskStatus::Running);
            manager.emit(TaskEvent {
                task_id: id.clone(),
                name,
                kind: TaskEventKind::Started,
                progress: None,
                error: None,
            });

            if cancel.is_cancelled() {
                finish_cancelled(&manager, &state, &id, name);
                return;
            }

            let blocking_result = tokio::task::spawn_blocking(f).await;

            if cancel.is_cancelled() {
                finish_cancelled(&manager, &state, &id, name);
                return;
            }

            match blocking_result {
                Ok(Ok(output)) => {
                    let _ = status_tx.send(TaskStatus::Completed);
                    manager
                        .inner
                        .registry
                        .set_status(&id, TaskStatus::Completed);
                    manager.emit(TaskEvent {
                        task_id: id.clone(),
                        name,
                        kind: TaskEventKind::Completed,
                        progress: None,
                        error: None,
                    });
                    if let Some(tx) = state.result_tx.lock().unwrap().take() {
                        let _ = tx.send(Ok(output));
                    }
                }
                Ok(Err(error)) => {
                    let message = error.to_string();
                    let _ = status_tx.send(TaskStatus::Failed);
                    manager.inner.registry.set_status(&id, TaskStatus::Failed);
                    manager.emit(TaskEvent {
                        task_id: id.clone(),
                        name,
                        kind: TaskEventKind::Failed,
                        progress: None,
                        error: Some(message),
                    });
                    if let Some(tx) = state.result_tx.lock().unwrap().take() {
                        let _ = tx.send(Err(error));
                    }
                }
                Err(error) => {
                    let message = error.to_string();
                    let _ = status_tx.send(TaskStatus::Failed);
                    manager.inner.registry.set_status(&id, TaskStatus::Failed);
                    manager.emit(TaskEvent {
                        task_id: id.clone(),
                        name,
                        kind: TaskEventKind::Failed,
                        progress: None,
                        error: Some(message.clone()),
                    });
                    if let Some(tx) = state.result_tx.lock().unwrap().take() {
                        let _ = tx.send(Err(TaskError::execution(message).into()));
                    }
                }
            }

            let _ = app;
        });

        *state_for_abort.abort.lock().unwrap() = Some(join.abort_handle());

        Ok(handle)
    }
}

fn finish_cancelled<O>(
    manager: &TaskManagerService,
    state: &SharedTaskState<O>,
    id: &TaskId,
    name: &'static str,
) {
    let _ = state.status.send(TaskStatus::Cancelled);
    manager.inner.registry.set_status(id, TaskStatus::Cancelled);
    manager.emit(TaskEvent {
        task_id: id.clone(),
        name,
        kind: TaskEventKind::Cancelled,
        progress: Some(state.progress_rx.borrow().clone()),
        error: None,
    });
    if let Some(tx) = state.result_tx.lock().unwrap().take() {
        let _ = tx.send(Err(TaskError::cancelled(format!(
            "task `{name}` cancelled"
        ))
        .into()));
    }
}
