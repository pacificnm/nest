//! eframe application shell and main loop handoff.

use std::sync::Arc;

use eframe::egui;
use nest_core::AppContext;
use nest_error::{NestError, NestResult};

use crate::codes::NEST_GUI_EFRAME_START_FAILED;
use crate::config::GuiRuntimeConfig;
use crate::render::render_in_app_error;
use crate::theme::apply_active_theme;
use crate::view::GuiView;

/// Runs the eframe main loop with the prepared Nest context and root view.
pub fn run_eframe(
    runtime: &GuiRuntimeConfig,
    ctx: Arc<AppContext>,
    view: Box<dyn GuiView>,
) -> NestResult<()> {
    let title = runtime.title.clone();
    let width = runtime.width;
    let height = runtime.height;
    let vsync = runtime.vsync;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(title)
            .with_inner_size([width as f32, height as f32]),
        vsync,
        ..Default::default()
    };

    let shell_ctx = ctx.clone();
    let result = eframe::run_native(
        &runtime.title,
        options,
        Box::new(move |cc| {
            cc.egui_ctx
                .style_mut(|style| {
                    let _ = apply_active_theme(&mut style.visuals, &shell_ctx);
                });
            Ok(Box::new(GuiShell {
                view,
                ctx: shell_ctx,
            }))
        }),
    );

    match result {
        Ok(()) => Ok(()),
        Err(error) => Err(
            NestError::ui(format!("eframe failed to start: {error}"))
                .with_code(NEST_GUI_EFRAME_START_FAILED),
        ),
    }
}

struct GuiShell {
    view: Box<dyn GuiView>,
    ctx: Arc<AppContext>,
}

impl eframe::App for GuiShell {
    fn update(&mut self, egui_ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(egui_ctx, |ui| {
            if let Err(error) = self.view.ui(ui, self.ctx.as_ref()) {
                render_in_app_error(ui, &error.report());
            }
        });
    }
}
