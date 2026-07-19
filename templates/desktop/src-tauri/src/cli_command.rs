// src/cli_command.rs

use serde::{Deserialize, Serialize};

/// Shared command enum for desktop ↔ CLI communication.
#[derive(Debug, Serialize, Deserialize)]
pub enum CliCommand {
    RunSystem { cmd: String, args: Vec<String> },
    HttpGet { url: String },
    AboutVersion,
}
