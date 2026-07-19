// src/main.rs

use nest_app::NestAppBuilder;
use nest_logging::init_logging;
use nest_logging::prelude::*; // re‑exports LoggingConfig
use nest_error::NestResult;

mod cli_command; // bring the enum into scope
use cli_command::CliCommand;

/// Entry point for the CLI binary when used directly (debugging).
fn main() -> NestResult<()> {
    // Initialise logging – you can customise the config later.
    init_logging(LoggingConfig::default())?;

    // Build a minimal Nest app – register any modules you need here.
    let _app = NestAppBuilder::new(env!("CARGO_PKG_NAME")).build()?;
    Ok(())
}

/// Public API that the desktop calls via IPC.
/// Returns a JSON‑serialisable string on success or a NestError on failure.
pub fn handle_cli_command(command: CliCommand) -> NestResult<String> {
    match command {
        CliCommand::RunSystem { cmd, args } => {
            // Run a system command and capture its output.
            let output = std::process::Command::new(&cmd)
                .args(&args)
                .output()
                .map_err(|e| nest_error::NestError::io(e.to_string()))?;
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            Ok(format!("out: {}\nerr: {}", stdout, stderr))
        }
        CliCommand::HttpGet { url } => {
            // Simple blocking GET request (reqwest is a dependency of nest-http).
            let body = reqwest::blocking::get(&url)
                .and_then(|resp| resp.text())
                .map_err(|e| nest_error::NestError::io(e.to_string()))?;
            Ok(body)
        }
    }
}
