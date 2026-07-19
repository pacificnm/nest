// src/cli_command.rs

use serde::{Deserialize, Serialize};

/// Commands the desktop can ask the CLI to execute.
#[derive(Debug, Serialize, Deserialize)]
pub enum CliCommand {
    /// Execute a system command and capture its output.
    RunSystem { cmd: String, args: Vec<String> },
    /// Perform a simple HTTP GET request.
    HttpGet { url: String },
    // Extend with more variants as needed.
}
