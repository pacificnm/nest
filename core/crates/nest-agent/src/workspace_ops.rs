//! Workspace build and verification helpers for agent tools.

use std::path::Path;
use std::process::{Command, Stdio};

use crate::{NestError, NestResult};

/// Runs `cargo check` in the workspace root and returns compiler output.
pub fn cargo_check(workspace_root: &Path, package: Option<&str>) -> NestResult<String> {
    if !workspace_root.join("Cargo.toml").is_file() {
        return Err(NestError::validation(format!(
            "no Cargo.toml in workspace root `{}`",
            workspace_root.display()
        )));
    }

    let mut command = Command::new("cargo");
    command
        .arg("check")
        .arg("--message-format=short")
        .current_dir(workspace_root)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if let Some(package) = package.filter(|value| !value.trim().is_empty()) {
        command.args(["-p", package.trim()]);
    }

    let output = command
        .output()
        .map_err(|error| NestError::io(format!("failed to run cargo check: {error}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}{stderr}").trim().to_string();

    if combined.is_empty() {
        if output.status.success() {
            return Ok("cargo check succeeded with no output.".into());
        }
        return Err(NestError::validation(
            "cargo check failed with no output (is cargo installed?)",
        ));
    }

    let summary = if output.status.success() {
        format!("cargo check succeeded.\n\n{combined}")
    } else {
        format!("cargo check failed (exit {}).\n\n{combined}", output.status)
    };

    Ok(truncate_output(&summary))
}

fn truncate_output(text: &str) -> String {
    const LIMIT: usize = 16_000;
    if text.len() <= LIMIT {
        text.to_string()
    } else {
        format!(
            "{}\n\n… (truncated, {} bytes total)",
            &text[..LIMIT],
            text.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn rejects_missing_manifest() {
        let dir = tempdir().unwrap();
        let error = cargo_check(dir.path(), None).unwrap_err().to_string();
        assert!(error.contains("no Cargo.toml"));
    }
}
