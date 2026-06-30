//! Desktop GUI host for the Nest framework.
//!
//! `nest-gui` wires modules, loads configuration, initializes file-only logging,
//! and runs an eframe/egui main loop.

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

pub mod codes;
pub mod prelude;

mod app;
mod bootstrap;
mod config;
mod logging;
mod module;
mod render;
mod shell;
mod startup;
mod theme;
mod view;

pub use app::GuiApp;
pub use bootstrap::PreparedRuntime;
pub use config::GuiRuntimeConfig;
pub use module::{GuiModule, GUI_MODULE_ID};
pub use startup::GuiStartupOptions;
pub use view::GuiView;

pub use nest_config::{ConfigDocument, ConfigLoader, ConfigService, ConfigSource};
pub use nest_core::{AppContext, Module, ModuleId};
pub use nest_error::{NestError, NestErrorReport, NestResult};
pub use nest_logging::LoggingConfig;

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use nest_config::ConfigDocument;
    use nest_logging::LoggingConfig;

    use super::*;

    static TEST_LOCK: Mutex<()> = Mutex::new(());

    fn integration_test_lock() -> std::sync::MutexGuard<'static, ()> {
        TEST_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    struct TestView;

    impl GuiView for TestView {
        fn ui(&mut self, ui: &mut egui::Ui, _ctx: &AppContext) -> NestResult<()> {
            ui.label("test");
            Ok(())
        }
    }

    #[test]
    fn startup_options_parse_window_flags() {
        let options = GuiStartupOptions::from_args_iter([
            "kiwi",
            "--config",
            "app.toml",
            "--title",
            "Kiwi IDE",
            "--width",
            "1024",
            "--height",
            "768",
            "--debug",
        ])
        .unwrap();
        assert_eq!(options.title.as_deref(), Some("Kiwi IDE"));
        assert_eq!(options.width, Some(1024));
        assert_eq!(options.height, Some(768));
        assert!(options.debug);
    }

    #[test]
    fn merge_runtime_config_respects_precedence() {
        let document =
            ConfigDocument::parse_toml("[gui]\nwidth = 800\nheight = 600\n").unwrap();
        let startup = GuiStartupOptions {
            width: Some(1920),
            height: Some(1080),
            ..GuiStartupOptions::default()
        };
        let runtime = crate::config::merge_runtime_config("kiwi", &document, &startup).unwrap();
        assert_eq!(runtime.width, 1920);
        assert_eq!(runtime.height, 1080);
    }

    #[test]
    fn logging_config_has_no_console() {
        let document = ConfigDocument::empty();
        let startup = GuiStartupOptions::default();
        let config = crate::logging::build_logging_config(
            LoggingConfig::for_gui("kiwi"),
            &document,
            &startup,
        )
        .unwrap();
        assert!(!config.has_console());
        assert!(config.has_file());
    }

    #[test]
    fn prepare_runtime_without_eframe() {
        let _guard = integration_test_lock();
        let mut prepared = GuiApp::new("kiwi")
            .view(TestView)
            .try_prepare_runtime(["kiwi"])
            .unwrap();
        assert_eq!(prepared.runtime_config.title, "kiwi");
        prepared.nest_app.shutdown().unwrap();
    }

    #[test]
    fn from_nest_app_skips_module_registration() {
        let _guard = integration_test_lock();
        use nest_app::NestApp;

        let nest_app = NestApp::builder("kiwi").build().unwrap();
        let mut prepared = GuiApp::from_nest_app(nest_app)
            .view(TestView)
            .try_prepare_runtime(["kiwi"])
            .unwrap();
        assert_eq!(prepared.nest_app.metadata().name, "kiwi");
        prepared.nest_app.shutdown().unwrap();
    }
}
