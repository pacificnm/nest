//! Terminal setup, restore, and panic safety.

use crossterm::event::DisableMouseCapture;
use crossterm::event::EnableMouseCapture;
use crossterm::execute;
use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use crossterm::terminal::EnterAlternateScreen;
use crossterm::terminal::LeaveAlternateScreen;
use nest_error::{NestError, NestResult};
use std::io::{self, stdout, Stdout, Write};
use std::panic;
use std::sync::OnceLock;

use crate::codes::NEST_TUI_TERMINAL_INIT_FAILED;
use crate::config::TuiRuntimeConfig;

static ACTIVE_CONFIG: OnceLock<TuiRuntimeConfig> = OnceLock::new();

/// RAII guard that restores the terminal on drop.
pub struct TerminalGuard {
    raw_mode: bool,
    alternate_screen: bool,
    mouse: bool,
}

impl TerminalGuard {
    /// Configures the terminal for TUI rendering.
    pub fn enter(config: &TuiRuntimeConfig) -> NestResult<Self> {
        let _ = ACTIVE_CONFIG.set(config.clone());

        if config.raw_mode {
            enable_raw_mode().map_err(|error| terminal_error("enable raw mode", error))?;
        }
        if config.alternate_screen {
            execute!(stdout(), EnterAlternateScreen)
                .map_err(|error| terminal_error("enter alternate screen", error))?;
        }
        if config.mouse {
            execute!(stdout(), EnableMouseCapture)
                .map_err(|error| terminal_error("enable mouse capture", error))?;
        }

        install_panic_hook();

        Ok(Self {
            raw_mode: config.raw_mode,
            alternate_screen: config.alternate_screen,
            mouse: config.mouse,
        })
    }

    /// Restores the terminal to its original state.
    pub fn restore(&self) -> NestResult<()> {
        restore_terminal_state(self.raw_mode, self.alternate_screen, self.mouse)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

/// Installs a panic hook that restores the terminal before printing the panic.
pub fn install_panic_hook() {
    let original = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal_best_effort();
        let _ = io::stderr().flush();
        let _ = io::stdout().flush();
        original(info);
    }));
}

/// Best-effort terminal restore using the last active runtime config.
pub fn restore_terminal_best_effort() -> NestResult<()> {
    if let Some(config) = ACTIVE_CONFIG.get() {
        restore_terminal_state(config.raw_mode, config.alternate_screen, config.mouse)
    } else {
        Ok(())
    }
}

fn restore_terminal_state(raw_mode: bool, alternate_screen: bool, mouse: bool) -> NestResult<()> {
    if mouse {
        let _ = execute!(stdout(), DisableMouseCapture);
    }
    if alternate_screen {
        let _ = execute!(stdout(), LeaveAlternateScreen);
    }
    if raw_mode {
        let _ = disable_raw_mode();
    }
    Ok(())
}

fn terminal_error(action: &str, error: io::Error) -> NestError {
    NestError::io(format!("failed to {action}: {error}")).with_code(NEST_TUI_TERMINAL_INIT_FAILED)
}

/// Type alias for the crossterm-backed ratatui terminal.
pub type TuiTerminal = ratatui::Terminal<ratatui::backend::CrosstermBackend<Stdout>>;

/// Creates a ratatui terminal using the crossterm backend.
pub fn create_terminal() -> NestResult<TuiTerminal> {
    let backend = ratatui::backend::CrosstermBackend::new(stdout());
    ratatui::Terminal::new(backend).map_err(|error| {
        NestError::io(format!("failed to create terminal: {error}"))
            .with_code(NEST_TUI_TERMINAL_INIT_FAILED)
    })
}
