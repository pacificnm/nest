//! Status bar message styling.

use egui::{Color32, Visuals};

/// Visual style for a status bar message.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum StatusKind {
    /// Neutral informational text.
    #[default]
    Info,
    /// In-progress work (shows spinner when busy).
    Loading,
    /// Successful completion.
    Success,
    /// Failure or error state.
    Error,
}

impl StatusKind {
    /// Text color for this kind against the current theme.
    pub fn text_color(self, visuals: &Visuals) -> Color32 {
        match self {
            Self::Info | Self::Loading => visuals.weak_text_color(),
            Self::Success => visuals.strong_text_color(),
            Self::Error => visuals.error_fg_color,
        }
    }
}

/// Connection indicator color for the right-side status dot.
pub fn connection_dot_color(connected: bool) -> Color32 {
    if connected {
        Color32::from_rgb(52, 168, 83)
    } else {
        Color32::from_rgb(234, 67, 53)
    }
}
