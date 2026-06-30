# nest-tui

Terminal UI host for the [Nest framework](../../README.md).

**Crate path:** [`core/crates/nest-tui`](../../core/crates/nest-tui)

`nest-tui` owns terminal lifecycle, startup option parsing, and the Ratatui event loop. It does not own CSV, HTTP, data, validation, or logging logic beyond host initialization.

## Quick start

```rust
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use nest_core::AppContext;
use nest_error::NestResult;
use nest_tui::{TuiAction, TuiApp, TuiScreen};
use ratatui::widgets::Paragraph;

struct MainScreen;

impl TuiScreen for MainScreen {
    fn draw(&mut self, frame: &mut ratatui::Frame, _ctx: &AppContext) -> NestResult<()> {
        frame.render_widget(Paragraph::new("Hello, Finch"), frame.area());
        Ok(())
    }

    fn on_event(&mut self, event: Event, _ctx: &AppContext) -> NestResult<TuiAction> {
        if let Event::Key(KeyEvent {
            kind: KeyEventKind::Press,
            code: KeyCode::Char('q'),
            modifiers: KeyModifiers::NONE,
            ..
        }) = event
        {
            return Ok(TuiAction::Quit);
        }
        Ok(TuiAction::Continue)
    }
}

fn main() {
    TuiApp::new("finch")
        .module(/* FileModule, etc. */)
        .screen(MainScreen)
        .run();
}
```

## Startup options

| Flag | Effect |
|------|--------|
| `--config <path>` | Load explicit config file |
| `--log-level <level>` | Override log level |
| `--log-file <path>` | Log file directory/path |
| `--no-color` | Plain error output on stderr |
| `--mouse` / `--no-mouse` | Mouse capture |
| `--tick-rate <ms>` | Event poll interval |
| `--debug` | Debug logging |

## `[tui]` config section

```toml
[tui]
mouse = true
tick_rate_ms = 250
alternate_screen = true
raw_mode = true
```

CLI flags override config; config overrides defaults.

## Logging

TUI hosts **never log to stdout** while the terminal UI is active. Default: file logging via `LoggingConfig::for_tui`.

## Related

- [Implementation plan](../plan/nest-tui-v1.md)
- [nest-cli](../nest-cli/README.md) — CLI host
- [nest-config](../nest-config/README.md) — configuration
