//! Renders the bottom status bar panel.

use egui::{Align, Color32, Layout, RichText, Sense, TopBottomPanel, Ui, Vec2};
use nest_core::AppContext;

use super::kind::connection_dot_color;
use super::service::{StatusBarRight, StatusBarService};

const DOT_RADIUS: f32 = 4.0;

/// Draws the status bar docked to the bottom of the window.
///
/// Call once per frame from the GUI shell before the central panel (with the
/// root [`egui::Context`]).
pub fn show_status_bar(ctx: &egui::Context, app_ctx: &AppContext) {
    let Ok(status) = app_ctx.service::<StatusBarService>() else {
        return;
    };

    let state = status.state();
    if !state.is_visible() {
        return;
    }

    let height = status.height();
    let message = state.message;
    let busy = state.busy;
    let kind = state.kind;
    let right = state.right;

    let panel_fill = ctx.style().visuals.panel_fill;

    TopBottomPanel::bottom("nest-status-bar")
        .exact_height(height)
        .show_separator_line(false)
        .frame(
            egui::Frame::new()
                .fill(panel_fill)
                .inner_margin(egui::Margin::ZERO),
        )
        .show(ctx, |ui| {
            let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
            let text_color = kind.text_color(ui.visuals());
            let weak_text = ui.visuals().weak_text_color();
            ui.painter().hline(
                ui.max_rect().x_range(),
                ui.max_rect().top(),
                stroke,
            );

            ui.horizontal(|ui| {
                ui.set_min_height(height);
                ui.spacing_mut().item_spacing.x = 8.0;

                ui.add_space(12.0);
                if !message.is_empty() {
                    if busy {
                        ui.spinner();
                    }
                    ui.label(RichText::new(message).size(12.0).color(text_color));
                }

                if right.is_some() {
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.set_width(ui.available_width());
                        ui.spacing_mut().item_spacing.x = 8.0;
                        ui.add_space(12.0);
                        if let Some(right) = &right {
                            render_right(ui, right, weak_text);
                        }
                    });
                }
            });
        });

    if busy {
        ctx.request_repaint();
    }
}

fn render_right(ui: &mut Ui, right: &StatusBarRight, text_color: Color32) {
    ui.label(
        RichText::new(&right.label)
            .size(12.0)
            .color(text_color),
    );
    if right.show_connection_dot {
        connection_dot(ui, right.connected);
    }
}

fn connection_dot(ui: &mut Ui, connected: bool) {
    let size = DOT_RADIUS * 2.0;
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    ui.painter()
        .circle_filled(rect.center(), DOT_RADIUS, connection_dot_color(connected));
    let status = if connected { "Connected" } else { "Disconnected" };
    response.on_hover_text(status);
}
