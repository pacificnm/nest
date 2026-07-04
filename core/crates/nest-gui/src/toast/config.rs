//! Toast display defaults.

use super::kind::ToastPosition;

/// Default toast behavior for [`super::ToastService`].
#[derive(Debug, Clone)]
pub struct ToastConfig {
    /// Corner placement for the toast stack.
    pub position: ToastPosition,
    /// Auto-dismiss duration in milliseconds.
    pub duration_ms: u64,
}

impl Default for ToastConfig {
    fn default() -> Self {
        Self {
            position: ToastPosition::default(),
            duration_ms: 5_000,
        }
    }
}

impl ToastConfig {
    /// Creates config with defaults (left bottom, 5s).
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets stack placement.
    pub fn position(mut self, position: ToastPosition) -> Self {
        self.position = position;
        self
    }

    /// Sets default auto-dismiss duration in milliseconds.
    pub fn duration_ms(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }
}
