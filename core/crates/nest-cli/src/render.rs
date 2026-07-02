//! CLI error rendering from [`NestErrorReport`].

use nest_error::{NestError, NestErrorReport};

use crate::globals::CliGlobals;

/// Renders an error for the terminal.
pub fn render_error(error: &NestError, globals: &CliGlobals) {
    let report = error.report();
    if globals.json {
        render_json(&report);
    } else {
        render_human(&report, globals);
    }
}

fn render_human(report: &NestErrorReport, globals: &CliGlobals) {
    let use_color = !globals.no_color;
    let code = report.code.as_deref().unwrap_or("NEST_UNKNOWN");

    if use_color {
        eprintln!("\x1b[1;31merror[{code}]\x1b[0m: {}", report.message);
    } else {
        eprintln!("error[{code}]: {}", report.message);
    }

    for detail in &report.details {
        eprintln!();
        eprintln!("  {detail}");
    }

    if let Some(help) = &report.help {
        eprintln!();
        eprintln!("  help: {help}");
    }
}

fn render_json(report: &NestErrorReport) {
    let payload = serde_json::json!({
        "success": false,
        "error": {
            "code": report.code,
            "kind": report.kind.label(),
            "message": report.message,
            "help": report.help,
            "details": report.details,
        }
    });
    eprintln!(
        "{}",
        serde_json::to_string_pretty(&payload).unwrap_or_else(|_| {
            serde_json::json!({"success": false, "error": {"message": report.message}}).to_string()
        })
    );
}
