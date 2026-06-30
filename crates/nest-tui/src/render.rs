//! Fatal error rendering for TUI hosts.

use nest_error::NestErrorReport;
use std::io::Write;

use crate::config::TuiRuntimeConfig;

/// Renders a fatal error to stderr before or after terminal restore.
pub fn render_error(report: &NestErrorReport, runtime: &TuiRuntimeConfig) {
    let mut stderr = std::io::stderr().lock();
    if runtime.no_color {
        let _ = writeln!(stderr, "error: {}", report.message);
        if let Some(code) = &report.code {
            let _ = writeln!(stderr, "code: {code}");
        }
        if let Some(help) = &report.help {
            let _ = writeln!(stderr, "help: {help}");
        }
        return;
    }

    let _ = writeln!(stderr, "\x1b[31merror:\x1b[0m {}", report.message);
    if let Some(code) = &report.code {
        let _ = writeln!(stderr, "\x1b[90mcode:\x1b[0m {code}");
    }
    if let Some(help) = &report.help {
        let _ = writeln!(stderr, "\x1b[90mhelp:\x1b[0m {help}");
    }
}

/// Renders a fatal error from a [`nest_error::NestError`].
pub fn render_nest_error(error: &nest_error::NestError, runtime: &TuiRuntimeConfig) {
    render_error(&error.report(), runtime);
}
