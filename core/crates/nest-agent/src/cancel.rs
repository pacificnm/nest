//! Agent cancellation handle.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Shared flag checked between agent steps.
#[derive(Clone, Default)]
pub struct CancelToken {
    inner: Arc<AtomicBool>,
}

impl CancelToken {
    /// Returns a token in the running state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Signals the agent loop to stop.
    pub fn cancel(&self) {
        self.inner.store(true, Ordering::SeqCst);
    }

    /// Returns true when cancellation was requested.
    pub fn is_cancelled(&self) -> bool {
        self.inner.load(Ordering::SeqCst)
    }
}
