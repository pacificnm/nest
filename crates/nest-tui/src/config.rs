//! `[tui]` configuration section and merge logic.

use nest_config::ConfigDocument;
use nest_error::NestResult;
use serde::Deserialize;

use crate::startup::TuiStartupOptions;

/// Runtime terminal settings after merging defaults, config file, and CLI flags.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TuiRuntimeConfig {
    /// Enable mouse capture.
    pub mouse: bool,
    /// Event poll / redraw interval in milliseconds.
    pub tick_rate_ms: u64,
    /// Use the terminal alternate screen buffer.
    pub alternate_screen: bool,
    /// Enable raw mode for keyboard input.
    pub raw_mode: bool,
    /// Disable ANSI color in host error output.
    pub no_color: bool,
}

impl Default for TuiRuntimeConfig {
    fn default() -> Self {
        Self {
            mouse: false,
            tick_rate_ms: 250,
            alternate_screen: true,
            raw_mode: true,
            no_color: false,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct TuiSection {
    mouse: Option<bool>,
    tick_rate_ms: Option<u64>,
    alternate_screen: Option<bool>,
    raw_mode: Option<bool>,
}

/// Merges TUI runtime settings: defaults < config file < CLI flags.
pub fn merge_runtime_config(
    document: &ConfigDocument,
    startup: &TuiStartupOptions,
) -> NestResult<TuiRuntimeConfig> {
    let mut config = TuiRuntimeConfig::default();

    if document.has_section("tui") {
        let section: TuiSection = document.section("tui")?;
        if let Some(mouse) = section.mouse {
            config.mouse = mouse;
        }
        if let Some(tick_rate_ms) = section.tick_rate_ms {
            config.tick_rate_ms = tick_rate_ms;
        }
        if let Some(alternate_screen) = section.alternate_screen {
            config.alternate_screen = alternate_screen;
        }
        if let Some(raw_mode) = section.raw_mode {
            config.raw_mode = raw_mode;
        }
    }

    if let Some(mouse) = startup.mouse {
        config.mouse = mouse;
    }
    if let Some(tick_rate_ms) = startup.tick_rate_ms {
        config.tick_rate_ms = tick_rate_ms;
    }
    if startup.no_color {
        config.no_color = true;
    }

    Ok(config)
}
