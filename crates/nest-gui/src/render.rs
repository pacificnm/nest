//! Fatal and in-app error rendering for GUI hosts.

use std::io::Write;

use nest_error::NestErrorReport;

use crate::config::GuiRuntimeConfig;

/// Renders a fatal error to stderr before the window opens.
pub fn render_error(report: &NestErrorReport, runtime: &GuiRuntimeConfig) {
    let mut stderr = std::io::stderr().lock();
    if runtime.no_color {
        let _ = writeln!(stderr, "error: {}", report.message);
        if let Some(code) = &report.code {
            let _ = writeln!(stderr, "code: {code}");
        }
        return;
    }

    let _ = writeln!(stderr, "\x1b[31merror:\x1b[0m {}", report.message);
    if let Some(code) = &report.code {
        let _ = writeln!(stderr, "\x1b[90mcode:\x1b[0m {code}");
    }
}

/// Renders a fatal error from a [`nest_error::NestError`].
pub fn render_nest_error(error: &nest_error::NestError, runtime: &GuiRuntimeConfig) {
    render_error(&error.report(), runtime);
}

/// Renders a runtime error inside an egui window (v1 stub).
pub fn render_in_app_error(ui: &mut egui::Ui, report: &NestErrorReport) {
    ui.colored_label(egui::Color32::RED, format!("Error: {}", report.message));
    if let Some(code) = &report.code {
        ui.label(format!("code: {code}"));
    }
    if let Some(help) = &report.help {
        ui.label(format!("help: {help}"));
    }
}
