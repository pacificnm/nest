// src/lib.rs

use serde::{Deserialize, Serialize};

/// Commands the desktop and CLI can use.
#[derive(Debug, Serialize, Deserialize)]
pub enum CliCommand {
    /// Execute a system command and capture its output.
    RunSystem { cmd: String, args: Vec<String> },
    /// Perform a simple HTTP GET request.
    HttpGet { url: String },
    /// Return the app version.
    AboutVersion,
    /// List recipes applied to this app.
    ListRecipes,
}

