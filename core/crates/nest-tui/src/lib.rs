//! Terminal UI host for the Nest framework.
//!
//! `nest-tui` wires modules, loads configuration, initializes file-only logging,
//! manages terminal lifecycle, and runs a Ratatui event loop.

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

pub mod codes;
pub mod prelude;

mod app;
mod bootstrap;
mod config;
mod event_loop;
mod logging;
mod module;
mod render;
mod screen;
mod startup;
mod terminal;
mod theme;

pub use app::TuiApp;
pub use bootstrap::PreparedRuntime;
pub use config::TuiRuntimeConfig;
pub use module::{TuiModule, TUI_MODULE_ID};
pub use screen::{TuiAction, TuiScreen};
pub use startup::TuiStartupOptions;

pub use nest_config::{ConfigDocument, ConfigLoader, ConfigService, ConfigSource};
pub use nest_core::{AppContext, Module, ModuleId};
pub use nest_error::{NestError, NestErrorReport, NestResult};
pub use nest_logging::LoggingConfig;

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
    use nest_config::ConfigDocument;
    use nest_logging::LoggingConfig;
    use ratatui::widgets::Paragraph;

    use super::*;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn integration_test_lock() -> std::sync::MutexGuard<'static, ()> {
        TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    struct TestScreen {
        quit: bool,
    }

    impl TuiScreen for TestScreen {
        fn draw(&mut self, frame: &mut ratatui::Frame, _ctx: &AppContext) -> NestResult<()> {
            frame.render_widget(Paragraph::new("test"), frame.area());
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
                self.quit = true;
                return Ok(TuiAction::Quit);
            }
            Ok(TuiAction::Continue)
        }
    }

    #[test]
    fn startup_options_parse_config_and_debug() {
        let options = TuiStartupOptions::from_args_iter([
            "finch",
            "--config",
            "custom.toml",
            "--debug",
            "--mouse",
            "--tick-rate",
            "100",
        ])
        .unwrap();
        assert_eq!(
            options.config_path.as_deref().and_then(|p| p.to_str()),
            Some("custom.toml")
        );
        assert!(options.debug);
        assert_eq!(options.mouse, Some(true));
        assert_eq!(options.tick_rate_ms, Some(100));
    }

    #[test]
    fn merge_runtime_config_respects_precedence() {
        let document = ConfigDocument::parse_toml(
            "[tui]\nmouse = false\ntick_rate_ms = 500\n",
        )
        .unwrap();
        let startup = TuiStartupOptions {
            mouse: Some(true),
            tick_rate_ms: Some(100),
            ..TuiStartupOptions::default()
        };
        let runtime = crate::config::merge_runtime_config(&document, &startup).unwrap();
        assert!(runtime.mouse);
        assert_eq!(runtime.tick_rate_ms, 100);
    }

    #[test]
    fn logging_config_has_no_console() {
        let document = ConfigDocument::empty();
        let startup = TuiStartupOptions::default();
        let config = crate::logging::build_logging_config(
            LoggingConfig::for_tui("finch"),
            &document,
            &startup,
        )
        .unwrap();
        assert!(!config.has_console());
        assert!(config.has_file());
    }

    #[test]
    fn prepare_runtime_without_terminal() {
        let _guard = integration_test_lock();
        let mut prepared = TuiApp::new("finch")
            .screen(TestScreen { quit: false })
            .try_prepare_runtime(["finch"])
            .unwrap();
        assert_eq!(prepared.runtime_config.tick_rate_ms, 250);
        prepared.nest_app.shutdown().unwrap();
    }

    #[test]
    fn from_nest_app_prepares_runtime() {
        let _guard = integration_test_lock();
        use nest_app::NestApp;

        let nest_app = NestApp::builder("finch").build().unwrap();
        let mut prepared = TuiApp::from_nest_app(nest_app)
            .screen(TestScreen { quit: false })
            .try_prepare_runtime(["finch"])
            .unwrap();
        assert_eq!(prepared.nest_app.metadata().name, "finch");
        prepared.nest_app.shutdown().unwrap();
    }
}
