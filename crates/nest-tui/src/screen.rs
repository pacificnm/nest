//! Root screen contract for TUI applications.

use crossterm::event::Event;
use nest_core::AppContext;
use nest_error::NestResult;
use ratatui::Frame;

/// Action returned from event handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiAction {
    /// Continue the event loop.
    Continue,
    /// Exit the application.
    Quit,
}

/// Root terminal screen for a TUI application.
pub trait TuiScreen: Send + 'static {
    /// Renders the screen into the given frame.
    fn draw(&mut self, frame: &mut Frame, ctx: &AppContext) -> NestResult<()>;

    /// Handles a terminal input event.
    fn on_event(&mut self, event: Event, ctx: &AppContext) -> NestResult<TuiAction>;
}
