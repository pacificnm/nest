use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use nest_error::NestResult;
use nest_tui::prelude::*;
use nest_tui::AppContext;
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use {{app_id_snake}}_core::greet;

/// The root TUI screen. It is a thin presentation layer: all domain logic is
/// delegated to `{{app_id_snake}}_core`.
pub struct MainScreen {
    message: String,
}

impl MainScreen {
    pub fn new() -> Self {
        Self {
            message: "Press 'g' to greet, 'q' to quit.".to_string(),
        }
    }
}

impl TuiScreen for MainScreen {
    fn draw(&mut self, frame: &mut Frame, _ctx: &AppContext) -> NestResult<()> {
        let paragraph = Paragraph::new(self.message.as_str());
        frame.render_widget(paragraph, frame.area());
        Ok(())
    }

    fn on_event(&mut self, event: Event, _ctx: &AppContext) -> NestResult<TuiAction> {
        if let Event::Key(KeyEvent {
            kind: KeyEventKind::Press,
            code,
            modifiers: KeyModifiers::NONE,
            ..
        }) = event
        {
            match code {
                KeyCode::Char('q') => return Ok(TuiAction::Quit),
                KeyCode::Char('g') => {
                    self.message = greet("World");
                }
                _ => {}
            }
        }
        Ok(TuiAction::Continue)
    }
}
