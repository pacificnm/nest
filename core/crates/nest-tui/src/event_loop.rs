//! Ratatui event and draw loop.

use std::time::{Duration, Instant};

use crossterm::event;
use nest_core::AppContext;
use nest_error::{NestError, NestResult};

use crate::codes::NEST_TUI_EVENT_LOOP_FAILED;
use crate::config::TuiRuntimeConfig;
use crate::screen::{TuiAction, TuiScreen};
use crate::terminal::TuiTerminal;

/// Runs the main TUI event loop until the screen returns [`TuiAction::Quit`].
pub fn run_event_loop(
    mut terminal: TuiTerminal,
    runtime: &TuiRuntimeConfig,
    screen: &mut dyn TuiScreen,
    ctx: &AppContext,
) -> NestResult<()> {
    let tick_rate = Duration::from_millis(runtime.tick_rate_ms);
    let mut last_tick = Instant::now();

    loop {
        terminal
            .draw(|frame| {
                if let Err(error) = screen.draw(frame, ctx) {
                    tracing::error!(%error, "screen draw failed");
                }
            })
            .map_err(event_loop_error)?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout).map_err(event_loop_error)? {
            let action = screen
                .on_event(event::read().map_err(event_loop_error)?, ctx)
                .map_err(event_loop_error)?;
            if action == TuiAction::Quit {
                break;
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    Ok(())
}

fn event_loop_error(error: impl std::fmt::Display) -> NestError {
    NestError::io(format!("TUI event loop failed: {error}")).with_code(NEST_TUI_EVENT_LOOP_FAILED)
}
