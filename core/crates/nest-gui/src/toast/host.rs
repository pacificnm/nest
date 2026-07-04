//! Renders active toasts as floating overlays.

use egui::{Align, Area, Color32, Frame, Id, Order, RichText, Ui, Vec2};
use nest_core::AppContext;

use super::kind::ToastKind;
use super::service::ToastService;

const TOAST_MIN_WIDTH: f32 = 280.0;
const TOAST_ESTIMATED_HEIGHT: f32 = 52.0;

/// Draws all active toasts for the current frame.
///
/// Call once per frame from the GUI shell after the main view (typically with
/// the root [`egui::Context`], not a nested [`Ui`]).
pub fn show_toasts(ctx: &egui::Context, app_ctx: &AppContext) {
    let Ok(toasts) = app_ctx.service::<ToastService>() else {
        return;
    };

    toasts.prune_expired();
    let active = toasts.active();
    if active.is_empty() {
        return;
    }

    let config = toasts.config();
    let top_down_stack = matches!(
        config.position,
        super::kind::ToastPosition::LeftBottom | super::kind::ToastPosition::RightBottom
    );

    let ordered: Vec<_> = if top_down_stack {
        active.iter().rev().collect()
    } else {
        active.iter().collect()
    };

    for (index, toast) in ordered.into_iter().enumerate() {
        let (anchor, offset) = config
            .position
            .layout(index as f32, TOAST_ESTIMATED_HEIGHT);

        Area::new(Id::new(("nest-toast", toast.id)))
            .order(Order::Foreground)
            .anchor(anchor, offset)
            .show(ctx, |ui| {
                render_toast(ui, toast, &mut |id| toasts.dismiss(id));
            });
    }

    ctx.request_repaint_after(std::time::Duration::from_millis(250));
}

fn render_toast(ui: &mut Ui, toast: &super::service::ToastMessage, dismiss: &mut dyn FnMut(u64)) {
    let (bg, fg) = toast.kind.colors(ui.visuals());

    Frame::NONE
        .fill(bg)
        .corner_radius(8.0)
        .inner_margin(egui::Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.set_min_width(TOAST_MIN_WIDTH);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                toast_icon(ui, toast.kind, fg);
                ui.with_layout(egui::Layout::left_to_right(Align::Center), |ui| {
                    ui.label(RichText::new(&toast.message).color(fg).size(14.0));
                });
                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .add(
                            egui::Button::new(RichText::new("✕").size(12.0).color(fg))
                                .frame(false)
                                .min_size(Vec2::new(20.0, 20.0)),
                        )
                        .clicked()
                    {
                        dismiss(toast.id);
                    }
                });
            });
        });
}

fn toast_icon(ui: &mut Ui, kind: ToastKind, color: Color32) {
    #[cfg(feature = "icons")]
    {
        nest_icon::font::ensure_installed(ui.ctx());
        let icon = kind.icon();
        ui.label(icon.rich_text(16.0).color(color));
    }
    #[cfg(not(feature = "icons"))]
    {
        ui.label(RichText::new(kind.glyph()).size(16.0).strong().color(color));
    }
}
