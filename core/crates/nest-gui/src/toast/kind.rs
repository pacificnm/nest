//! Toast severity and placement.

use egui::{Align2, Color32, Vec2, Visuals};

/// Toast severity — maps to color and icon.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    /// Operation succeeded.
    Success,
    /// Non-blocking warning.
    Warning,
    /// Operation failed.
    Error,
    /// Informational message.
    Info,
}

impl ToastKind {
    /// Background and foreground colors for this kind.
    pub fn colors(self, visuals: &Visuals) -> (Color32, Color32) {
        match self {
            Self::Success => (
                Color32::from_rgb(34, 120, 70),
                Color32::WHITE,
            ),
            Self::Warning => (
                Color32::from_rgb(180, 120, 20),
                Color32::WHITE,
            ),
            Self::Error => (
                visuals.error_fg_color,
                Color32::WHITE,
            ),
            Self::Info => (
                visuals.hyperlink_color,
                Color32::WHITE,
            ),
        }
    }

    #[cfg(feature = "icons")]
    /// Font Awesome icon for this toast kind.
    pub fn icon(self) -> nest_icon::Icon {
        use nest_icon::Icon;
        match self {
            Self::Success => Icon::CHECK,
            Self::Warning => Icon::WARNING,
            Self::Error => Icon::ERROR,
            Self::Info => Icon::INFO,
        }
    }

    #[cfg(not(feature = "icons"))]
    /// Fallback text glyph when the `icons` feature is disabled.
    pub fn glyph(self) -> &'static str {
        match self {
            Self::Success => "✓",
            Self::Warning => "!",
            Self::Error => "✕",
            Self::Info => "i",
        }
    }
}

/// Screen corner / edge placement for the toast stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToastPosition {
    /// Bottom-left (default).
    #[default]
    LeftBottom,
    /// Top-left.
    LeftTop,
    /// Bottom-right.
    RightBottom,
    /// Top-right.
    RightTop,
}

impl ToastPosition {
    /// Anchor point and stack direction for overlay positioning.
    pub fn layout(self, stack_index: f32, toast_height: f32) -> (Align2, Vec2) {
        let margin = 16.0;
        let gap = 8.0;
        let stack_offset = stack_index * (toast_height + gap);

        match self {
            Self::LeftBottom => (
                Align2::LEFT_BOTTOM,
                Vec2::new(margin, -margin - stack_offset),
            ),
            Self::LeftTop => (
                Align2::LEFT_TOP,
                Vec2::new(margin, margin + stack_offset),
            ),
            Self::RightBottom => (
                Align2::RIGHT_BOTTOM,
                Vec2::new(-margin, -margin - stack_offset),
            ),
            Self::RightTop => (
                Align2::RIGHT_TOP,
                Vec2::new(-margin, margin + stack_offset),
            ),
        }
    }
}
