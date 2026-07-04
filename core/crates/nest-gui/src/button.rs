//! Icon + label action buttons with consistent sizing.

use egui::text::{LayoutJob, TextFormat};
use egui::{Button, Color32, FontFamily, FontId, Response, Ui, Vec2, Widget, WidgetText};
use nest_icon::{font, Icon};

/// Preset sizes for [`ActionButton`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ButtonSize {
    /// Compact toolbar control (72×28).
    Small,
    /// Default detail-page action (96×34) — matches Loon Play/Edit.
    #[default]
    Medium,
    /// Prominent primary action (128×40).
    Large,
    /// Full width of the available horizontal space.
    Full,
}

impl ButtonSize {
    fn dimensions(self) -> (f32, f32, f32, f32) {
        // (min_width, min_height, icon_size, text_size)
        match self {
            Self::Small => (72.0, 28.0, 12.0, 13.0),
            Self::Medium => (96.0, 34.0, 14.0, 14.0),
            Self::Large => (128.0, 40.0, 16.0, 16.0),
            Self::Full => (0.0, 38.0, 15.0, 15.0),
        }
    }
}

/// Icon + text button with Nest-standard sizing and optional fill color.
pub struct ActionButton {
    icon: Icon,
    label: String,
    size: ButtonSize,
    fill: Option<Color32>,
    text_color: Option<Color32>,
    enabled: bool,
    tooltip: Option<String>,
}

impl ActionButton {
    /// Creates a button with an icon and action label.
    pub fn new(icon: Icon, label: impl Into<String>) -> Self {
        Self {
            icon,
            label: label.into(),
            size: ButtonSize::default(),
            fill: None,
            text_color: None,
            enabled: true,
            tooltip: None,
        }
    }

    /// Sets button size (default [`ButtonSize::Medium`]).
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Sets background fill color (e.g. theme primary for Play).
    pub fn fill(mut self, color: Color32) -> Self {
        self.fill = Some(color);
        self
    }

    /// Sets label and icon text color.
    pub fn text_color(mut self, color: Color32) -> Self {
        self.text_color = Some(color);
        self
    }

    /// Enables or disables interaction.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Sets hover tooltip text.
    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    fn widget_text(&self, ui: &Ui) -> WidgetText {
        let (_, _, icon_size, text_size) = self.size.dimensions();
        let color = self
            .text_color
            .unwrap_or_else(|| ui.visuals().text_color());

        let mut job = LayoutJob::default();
        job.append(
            &self.icon.glyph().to_string(),
            0.0,
            TextFormat {
                font_id: FontId::new(icon_size, self.icon.style().font_family()),
                color,
                ..Default::default()
            },
        );
        job.append(
            "  ",
            0.0,
            TextFormat {
                font_id: FontId::new(text_size, FontFamily::Proportional),
                color,
                ..Default::default()
            },
        );
        job.append(
            self.label.as_str(),
            0.0,
            TextFormat {
                font_id: FontId::new(text_size, FontFamily::Proportional),
                color,
                ..Default::default()
            },
        );
        job.into()
    }

    fn min_size(&self, ui: &Ui) -> Vec2 {
        let (min_width, min_height, _, _) = self.size.dimensions();
        let width = if self.size == ButtonSize::Full {
            ui.available_width()
        } else {
            min_width
        };
        Vec2::new(width.max(1.0), min_height)
    }
}

impl Widget for ActionButton {
    fn ui(self, ui: &mut Ui) -> Response {
        font::ensure_installed(ui.ctx());

        let corner_radius = ui.style().visuals.widgets.inactive.corner_radius;
        let mut button = Button::new(self.widget_text(ui))
            .min_size(self.min_size(ui))
            .corner_radius(corner_radius);

        if let Some(fill) = self.fill {
            button = button.fill(fill);
        }

        let response = ui.add_enabled(self.enabled, button);
        if let Some(tooltip) = self.tooltip {
            response.on_hover_text(tooltip)
        } else {
            response
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn medium_matches_loon_play_dimensions() {
        let (w, h, icon, text) = ButtonSize::Medium.dimensions();
        assert_eq!((w, h), (96.0, 34.0));
        assert_eq!(icon, 14.0);
        assert_eq!(text, 14.0);
    }

    #[test]
    fn default_size_is_medium() {
        assert_eq!(ActionButton::new(Icon::PLAY, "Play").size, ButtonSize::Medium);
    }
}
