//! Desktop host for the Nest framework using Tauri.
//!
//! `nest-tauri` wires modules, loads configuration, initializes file-only logging,
//! and runs a Tauri webview. Product UI lives in a separate `ui/` React app.

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

pub mod codes;
pub mod prelude;

mod app;
mod bootstrap;
#[cfg(feature = "runtime")]
mod commands;
mod config;
#[cfg(feature = "images")]
mod image;
mod logging;
mod module;
mod startup;
#[cfg(feature = "runtime")]
mod state;

#[cfg(feature = "runtime")]
pub use app::BuilderExtensionFn;
pub use app::TauriApp;
pub use bootstrap::PreparedRuntime;
pub use config::TauriRuntimeConfig;
pub use module::{TauriModule, TAURI_MODULE_ID};
pub use startup::TauriStartupOptions;

#[cfg(feature = "runtime")]
pub use state::NestHostState;

#[cfg(feature = "runtime")]
pub use commands::{AppMetadataResponse, ThemeCssResponse, ThemeSetActiveRequest, ThemeSummary};

#[cfg(feature = "images")]
pub use image::{
    fetch_image, invalidate_image_tag, ImageFetchRequest, ImageFetchResponse,
    ImageInvalidateTagRequest, ImageInvalidateTagResponse,
};

pub use nest_config::{ConfigDocument, ConfigLoader, ConfigService, ConfigSource};
pub use nest_core::{AppContext, Module, ModuleId};
pub use nest_error::{NestError, NestErrorReport, NestResult};
pub use nest_logging::LoggingConfig;
pub use nest_react_theme::{CssTheme, ReactThemeAdapter};

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

    #[test]
    fn startup_options_parse_window_flags() {
        let options = TauriStartupOptions::from_args_iter([
            "my-app", "--config", "app.toml", "--title", "My App", "--width", "1024", "--height",
            "768", "--debug",
        ])
        .unwrap();
        assert_eq!(options.title.as_deref(), Some("My App"));
        assert_eq!(options.width, Some(1024));
        assert_eq!(options.height, Some(768));
        assert!(options.debug);
    }

    #[test]
    fn merge_runtime_config_respects_precedence() {
        let document = ConfigDocument::parse_toml("[tauri]\nwidth = 800\nheight = 600\n").unwrap();
        let startup = TauriStartupOptions {
            width: Some(1920),
            height: Some(1080),
            ..TauriStartupOptions::default()
        };
        let runtime = crate::config::merge_runtime_config("my-app", &document, &startup).unwrap();
        assert_eq!(runtime.width, 1920);
        assert_eq!(runtime.height, 1080);
    }

    #[test]
    fn logging_config_has_no_console() {
        let document = ConfigDocument::empty();
        let startup = TauriStartupOptions::default();
        let config = crate::logging::build_logging_config(
            LoggingConfig::for_tauri("my-app"),
            &document,
            &startup,
        )
        .unwrap();
        assert!(!config.has_console());
        assert!(config.has_file());
    }

    #[test]
    fn prepare_runtime_without_tauri_loop() {
        let _guard = integration_test_lock();
        let mut prepared = TauriApp::new("my-app")
            .try_prepare_runtime(["my-app"])
            .unwrap();
        assert_eq!(prepared.runtime_config.title, "my-app");
        prepared.nest_app.shutdown().unwrap();
    }

    #[test]
    fn from_nest_app_skips_module_registration() {
        let _guard = integration_test_lock();
        use nest_app::NestApp;

        let nest_app = NestApp::builder("my-app").build().unwrap();
        let mut prepared = TauriApp::from_nest_app(nest_app)
            .try_prepare_runtime(["my-app"])
            .unwrap();
        assert_eq!(prepared.nest_app.metadata().name, "my-app");
        prepared.nest_app.shutdown().unwrap();
    }
}
