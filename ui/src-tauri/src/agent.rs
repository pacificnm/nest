//! PTY-backed AI agent runtime for the Nest Agent window.
//!
//! Runs an external coding agent (Claude Code, Codex, OpenCode, …) launched via
//! `ollama launch <runtime> --model <model>` (or the CLI binary directly, in
//! "account" mode) inside a pseudo-terminal. Output streams to the webview as
//! base64 chunks over [`AGENT_OUTPUT_EVENT`]; keystrokes flow back through
//! `main.rs` commands. Ported from Kiwi's `agent.rs`.

use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use base64::Engine;
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime};

use crate::nest_root::resolve_nest_root;

/// Event carrying a base64-encoded chunk of agent PTY output.
pub const AGENT_OUTPUT_EVENT: &str = "nest://agent-output";
/// Event fired once when the agent process exits.
pub const AGENT_EXIT_EVENT: &str = "nest://agent-exit";

/// Payload for [`AGENT_OUTPUT_EVENT`].
#[derive(Clone, Serialize)]
pub struct AgentOutput {
    pub base64: String,
}

/// Payload for [`AGENT_EXIT_EVENT`].
#[derive(Clone, Serialize)]
pub struct AgentExit {
    pub message: String,
}

/// A running agent PTY session.
struct Session {
    master: Box<dyn MasterPty + Send>,
    writer: Box<dyn Write + Send>,
}

/// Managed Tauri state holding at most one live agent session.
#[derive(Default)]
pub struct AgentPty {
    session: Arc<Mutex<Option<Session>>>,
}

impl AgentPty {
    /// Spawns `ollama launch <runtime> --model <model>` (or the runtime CLI
    /// directly, when `direct` is set) in a PTY.
    ///
    /// Replaces any existing session. Output streams to `app` via
    /// [`AGENT_OUTPUT_EVENT`]; exit is signalled with [`AGENT_EXIT_EVENT`].
    #[allow(clippy::too_many_arguments)]
    pub fn launch<R: Runtime>(
        &self,
        app: AppHandle<R>,
        runtime: &str,
        model: &str,
        ollama_host: Option<&str>,
        cwd: Option<PathBuf>,
        direct: bool,
        rows: u16,
        cols: u16,
    ) -> Result<(), String> {
        self.stop();

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|error| format!("failed to open pty: {error}"))?;

        // Two connection modes:
        //  * Ollama: `ollama launch <rt> --model <model>` routes the agent
        //            through the Ollama server (OLLAMA_HOST).
        //  * Direct: run the agent CLI itself (`claude`, `codex`, …) with no
        //            base-URL override, using its own signed-in account.
        let (mut command, describe) = if direct {
            (
                direct_command(runtime),
                format!("{} (account)", direct_binary(runtime)),
            )
        } else {
            let mut command = CommandBuilder::new("ollama");
            command.arg("launch");
            command.arg(runtime);
            command.arg("--model");
            command.arg(model);
            (command, format!("ollama launch {runtime} --model {model}"))
        };

        let workspace = cwd.filter(|dir| dir.is_dir());
        configure_agent_mcp_env(&mut command, runtime);
        if let Some(dir) = &workspace {
            command.cwd(dir);
        }
        command.env("TERM", "xterm-256color");
        command.env("COLORTERM", "truecolor");
        command.env("PATH", augmented_path());
        if let Ok(home) = std::env::var("HOME") {
            command.env("HOME", home);
        }
        // Only in Ollama mode do we point the CLI at the inference server. In
        // direct mode, leave OLLAMA_HOST unset so the agent uses its native
        // account + endpoint.
        if !direct {
            if let Some(host) = ollama_host.map(str::trim).filter(|h| !h.is_empty()) {
                command.env("OLLAMA_HOST", normalize_ollama_host(host));
            }
        }

        tracing::info!(
            target: "nest-agent",
            runtime,
            model,
            direct,
            ollama_host = ollama_host.unwrap_or("(local)"),
            "launching agent: {describe}"
        );

        let mut child = pair.slave.spawn_command(command).map_err(|error| {
            let message = if direct {
                format!(
                    "failed to launch `{}`: {error}. Is it installed and on PATH?",
                    direct_binary(runtime)
                )
            } else {
                format!(
                    "failed to launch `ollama launch {runtime} --model {model}`: {error}. \
                     Is Ollama v0.15+ installed and on PATH?"
                )
            };
            tracing::error!(target: "nest-agent", %error, runtime, direct, "agent launch failed");
            message
        })?;
        drop(pair.slave);

        let master = pair.master;
        let reader = master
            .try_clone_reader()
            .map_err(|error| format!("failed to clone pty reader: {error}"))?;
        let writer = master
            .take_writer()
            .map_err(|error| format!("failed to open pty writer: {error}"))?;

        *self.session.lock().expect("agent pty mutex") = Some(Session { master, writer });

        let out_app = app.clone();
        thread::spawn(move || read_loop(reader, out_app));

        let exit_app = app.clone();
        let session_slot = Arc::clone(&self.session);
        thread::spawn(move || {
            let message = match child.wait() {
                Ok(status) => format!("agent exited ({status})"),
                Err(error) => format!("agent wait failed: {error}"),
            };
            *session_slot.lock().expect("agent pty mutex") = None;
            let _ = exit_app.emit(AGENT_EXIT_EVENT, AgentExit { message });
        });

        tracing::info!(target: "nest-agent", runtime, model, rows, cols, "agent pty launched");
        Ok(())
    }

    /// Writes UTF-8 input (keystrokes) to the agent stdin.
    pub fn input(&self, data: &str) -> Result<(), String> {
        let mut guard = self.session.lock().expect("agent pty mutex");
        let session = guard
            .as_mut()
            .ok_or_else(|| "no agent session is running".to_string())?;
        session
            .writer
            .write_all(data.as_bytes())
            .map_err(|error| format!("failed to write to agent pty: {error}"))
    }

    /// Resizes the PTY grid to match the terminal viewport.
    pub fn resize(&self, rows: u16, cols: u16) -> Result<(), String> {
        let guard = self.session.lock().expect("agent pty mutex");
        if let Some(session) = guard.as_ref() {
            session
                .master
                .resize(PtySize {
                    rows,
                    cols,
                    pixel_width: 0,
                    pixel_height: 0,
                })
                .map_err(|error| format!("failed to resize agent pty: {error}"))?;
        }
        Ok(())
    }

    /// Terminates the running session, if any.
    pub fn stop(&self) {
        *self.session.lock().expect("agent pty mutex") = None;
    }

    /// Returns whether a session is currently running.
    pub fn is_running(&self) -> bool {
        self.session.lock().expect("agent pty mutex").is_some()
    }
}

/// Maps an `ollama launch` runtime id to its native CLI binary for direct mode.
fn direct_binary(runtime: &str) -> &str {
    match runtime {
        "claude" => "claude",
        "codex" | "codex-app" => "codex",
        other => other,
    }
}

/// Builds the command that runs an agent CLI directly (native account mode).
fn direct_command(runtime: &str) -> CommandBuilder {
    CommandBuilder::new(direct_binary(runtime))
}

/// Builds a `PATH` that includes common CLI install dirs.
///
/// GUI apps launched outside a login shell often inherit a minimal `PATH`
/// (e.g. `/usr/bin`), so `ollama` / agent binaries in `/usr/local/bin`,
/// Homebrew, or `~/.local/bin` are missing. Append the usual locations.
pub(crate) fn augmented_path() -> String {
    let mut parts: Vec<String> = std::env::var("PATH")
        .map(|path| {
            std::env::split_paths(&path)
                .map(|p| p.to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();

    let mut extras: Vec<String> = vec![
        "/usr/local/bin".into(),
        "/usr/bin".into(),
        "/bin".into(),
        "/opt/homebrew/bin".into(),
        "/snap/bin".into(),
    ];
    if let Ok(home) = std::env::var("HOME") {
        extras.push(format!("{home}/.local/bin"));
        extras.push(format!("{home}/bin"));
    }

    for extra in extras {
        if !parts.iter().any(|p| p == &extra) {
            parts.push(extra);
        }
    }
    parts.join(":")
}

/// Ensures the Ollama host has a scheme so agents that expect a URL work.
fn normalize_ollama_host(host: &str) -> String {
    if host.starts_with("http://") || host.starts_with("https://") {
        host.to_string()
    } else {
        format!("http://{host}")
    }
}

/// Configures MCP environment for external agent CLIs.
///
/// Nest Desktop's agent always runs inside the Nest monorepo, which already
/// hosts `.mcp.json` / `opencode.json` at its root — unlike Kiwi (a
/// per-project workspace tool), there's no nested-workspace root to resolve or
/// config file to copy. Claude and OpenCode both expand `NEST_PROJECT_ROOT`
/// from their respective root config files.
fn configure_agent_mcp_env(command: &mut CommandBuilder, runtime: &str) {
    let Ok(root) = resolve_nest_root() else {
        tracing::warn!(target: "nest-agent", "no Nest repository root found for MCP env");
        return;
    };
    command.env("NEST_PROJECT_ROOT", root.to_string_lossy().into_owned());

    if runtime == "opencode" {
        let config = root.join("opencode.json");
        if config.is_file() {
            command.env("OPENCODE_CONFIG", config.to_string_lossy().into_owned());
        }
    }
}

fn read_loop<R: Runtime>(mut reader: Box<dyn Read + Send>, app: AppHandle<R>) {
    let engine = base64::engine::general_purpose::STANDARD;
    let mut buffer = [0_u8; 8192];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => {
                let base64 = engine.encode(&buffer[..count]);
                if app
                    .emit(AGENT_OUTPUT_EVENT, AgentOutput { base64 })
                    .is_err()
                {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}
