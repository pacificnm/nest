//! Native agent-account helpers (direct connection mode).
//!
//! For the "account" connection mode, coding agents authenticate with their own
//! provider instead of the Ollama server. Codex exposes a clean CLI login flow
//! (`codex login`); Claude Code signs in interactively inside its own TUI, so we
//! only surface a hint for it in the UI. Ported from Kiwi's `accounts.rs`.

use std::process::{Command, Stdio};

use serde::Serialize;

use crate::agent::augmented_path;

/// Sign-in status for a native agent account.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountStatus {
    /// True when the CLI reports an authenticated session.
    pub signed_in: bool,
    /// Raw status line from the CLI (e.g. `Logged in using ChatGPT`).
    pub detail: String,
}

/// Runs `codex login status` and parses whether an account is signed in.
#[tauri::command]
pub fn codex_account_status() -> AccountStatus {
    let output = Command::new("codex")
        .args(["login", "status"])
        .env("PATH", augmented_path())
        .output();

    match output {
        Ok(output) => {
            let text = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let detail = text
                .lines()
                .chain(stderr.lines())
                .map(str::trim)
                .find(|line| !line.is_empty())
                .unwrap_or("Unknown")
                .to_string();
            let signed_in = !detail.to_lowercase().contains("not logged in");
            AccountStatus { signed_in, detail }
        }
        Err(error) => AccountStatus {
            signed_in: false,
            detail: format!("codex not available: {error}"),
        },
    }
}

/// Launches `codex login` (opens the browser OAuth flow) detached.
#[tauri::command]
pub fn codex_login() -> Result<(), String> {
    Command::new("codex")
        .arg("login")
        .env("PATH", augmented_path())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("failed to start `codex login`: {error}"))
}

/// Runs `codex logout` to clear stored credentials.
#[tauri::command]
pub fn codex_logout() -> Result<(), String> {
    let output = Command::new("codex")
        .arg("logout")
        .env("PATH", augmented_path())
        .output()
        .map_err(|error| format!("failed to run `codex logout`: {error}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(if stderr.is_empty() {
            "codex logout failed".to_string()
        } else {
            stderr
        })
    }
}
