//! Toast message bus.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::config::ToastConfig;
use super::kind::{ToastKind, ToastPosition};

/// A toast waiting to be shown or currently visible.
#[derive(Debug, Clone)]
pub struct ToastMessage {
    /// Stable id for dismiss and egui widget identity.
    pub id: u64,
    /// Severity / styling.
    pub kind: ToastKind,
    /// User-visible text.
    pub message: String,
    /// When the toast was enqueued.
    pub created_at: Instant,
    /// Auto-dismiss after this many milliseconds.
    pub duration_ms: u64,
}

/// Application-wide toast queue (register via [`super::ToastModule`]).
#[derive(Clone)]
pub struct ToastService {
    inner: Arc<ToastInner>,
}

struct ToastInner {
    config: Mutex<ToastConfig>,
    toasts: Mutex<Vec<ToastMessage>>,
    next_id: AtomicU64,
}

impl ToastService {
    /// Creates a service with the given defaults.
    pub fn new(config: ToastConfig) -> Self {
        Self {
            inner: Arc::new(ToastInner {
                config: Mutex::new(config),
                toasts: Mutex::new(Vec::new()),
                next_id: AtomicU64::new(1),
            }),
        }
    }

    /// Current configuration (placement and default duration).
    pub fn config(&self) -> ToastConfig {
        self.inner.config.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Updates stack placement.
    pub fn set_position(&self, position: ToastPosition) {
        if let Ok(mut config) = self.inner.config.lock() {
            config.position = position;
        }
    }

    /// Updates default auto-dismiss duration in milliseconds.
    pub fn set_default_duration_ms(&self, duration_ms: u64) {
        if let Ok(mut config) = self.inner.config.lock() {
            config.duration_ms = duration_ms;
        }
    }

    /// Enqueues a toast using the configured default duration.
    pub fn show(&self, message: impl Into<String>, kind: ToastKind) -> u64 {
        self.show_for(message, kind, None)
    }

    /// Enqueues a toast with an optional custom duration (`None` = default).
    pub fn show_for(
        &self,
        message: impl Into<String>,
        kind: ToastKind,
        duration_ms: Option<u64>,
    ) -> u64 {
        let default = self.config().duration_ms;
        let id = self.inner.next_id.fetch_add(1, Ordering::Relaxed);
        let toast = ToastMessage {
            id,
            kind,
            message: message.into(),
            created_at: Instant::now(),
            duration_ms: duration_ms.unwrap_or(default),
        };

        if let Ok(mut toasts) = self.inner.toasts.lock() {
            toasts.push(toast);
        }
        id
    }

    /// Enqueues a success toast.
    pub fn success(&self, message: impl Into<String>) -> u64 {
        self.show(message, ToastKind::Success)
    }

    /// Enqueues a warning toast.
    pub fn warning(&self, message: impl Into<String>) -> u64 {
        self.show(message, ToastKind::Warning)
    }

    /// Enqueues an error toast.
    pub fn error(&self, message: impl Into<String>) -> u64 {
        self.show(message, ToastKind::Error)
    }

    /// Enqueues an info toast.
    pub fn info(&self, message: impl Into<String>) -> u64 {
        self.show(message, ToastKind::Info)
    }

    /// Removes a toast by id (manual dismiss).
    pub fn dismiss(&self, id: u64) {
        if let Ok(mut toasts) = self.inner.toasts.lock() {
            toasts.retain(|toast| toast.id != id);
        }
    }

    /// Removes expired toasts. Called each frame by the toast host.
    pub fn prune_expired(&self) {
        if let Ok(mut toasts) = self.inner.toasts.lock() {
            toasts.retain(|toast| {
                toast.created_at.elapsed() < Duration::from_millis(toast.duration_ms)
            });
        }
    }

    /// Active toasts for rendering (oldest first).
    pub fn active(&self) -> Vec<ToastMessage> {
        self.inner
            .toasts
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn show_and_dismiss() {
        let service = ToastService::new(ToastConfig::default());
        let id = service.success("Saved");
        assert_eq!(service.active().len(), 1);
        service.dismiss(id);
        assert!(service.active().is_empty());
    }

    #[test]
    fn prune_expires_old_toasts() {
        let service = ToastService::new(ToastConfig::default().duration_ms(1));
        service.info("hello");
        std::thread::sleep(Duration::from_millis(5));
        service.prune_expired();
        assert!(service.active().is_empty());
    }
}
