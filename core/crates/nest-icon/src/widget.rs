//! Icon button widget.

use egui::{Button, Response, Ui, Widget};

use crate::font;
use crate::Icon;

/// Button that displays a Font Awesome icon.
pub struct IconButton {
    icon: Icon,
    size: f32,
    tooltip: Option<String>,
    min_size: Option<egui::Vec2>,
}

impl IconButton {
    /// Creates an icon button.
    pub fn new(icon: Icon) -> Self {
        Self {
            icon,
            size: 16.0,
            tooltip: None,
            min_size: None,
        }
    }

    /// Sets icon size in points (default 16).
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Sets hover tooltip text.
    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Sets minimum button size.
    pub fn min_size(mut self, size: egui::Vec2) -> Self {
        self.min_size = Some(size);
        self
    }
}

impl Widget for IconButton {
    fn ui(self, ui: &mut Ui) -> Response {
        font::ensure_installed(ui.ctx());

        if !font::family_available(ui.ctx(), self.icon.style()) {
            ui.ctx().request_repaint();
            let mut button = Button::new(egui::RichText::new(" ").size(self.size));
            if let Some(min_size) = self.min_size {
                button = button.min_size(min_size);
            }
            return ui.add(button);
        }

        let mut button = Button::new(self.icon.rich_text(self.size));
        if let Some(min_size) = self.min_size {
            button = button.min_size(min_size);
        }

        let response = ui.add(button);
        if let Some(tooltip) = self.tooltip {
            response.on_hover_text(tooltip)
        } else {
            response
        }
    }
}
