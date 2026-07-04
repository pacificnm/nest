//! Status bar display defaults.

/// Default status bar behavior for [`super::StatusBarService`].
#[derive(Debug, Clone)]
pub struct StatusBarConfig {
    /// Fixed height of the bottom bar in points.
    pub height: f32,
}

impl Default for StatusBarConfig {
    fn default() -> Self {
        Self { height: 26.0 }
    }
}

impl StatusBarConfig {
    /// Creates config with defaults.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets bar height in points.
    pub fn height(mut self, height: f32) -> Self {
        self.height = height;
        self
    }
}
