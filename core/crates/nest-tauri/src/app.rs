//! Tauri application builder.

use std::ffi::OsString;
use std::path::PathBuf;

use nest_app::NestApp;
use nest_core::Module;
use nest_error::NestResult;
use nest_logging::LoggingConfig;

use crate::bootstrap::prepare_runtime;

#[cfg(feature = "runtime")]
use crate::bootstrap::{exit_with_error, run_with_context};

/// Extends the Tauri builder after Nest's built-in IPC handler is attached.
///
/// The closure receives the builder with [`crate::NestHostState`] already
/// managed and the built-in commands registered, so it must **not** call
/// `invoke_handler` (that would overwrite the built-ins). Instead, register
/// application state with `manage` and application IPC commands as a Tauri
/// **plugin** (`tauri::plugin::Builder::new(name).invoke_handler(...)`), whose
/// commands the UI invokes as `plugin:<name>|<command>`.
#[cfg(feature = "runtime")]
pub type BuilderExtensionFn =
    Box<dyn FnOnce(tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> + Send>;

/// Desktop Tauri host for Nest applications.
pub struct TauriApp {
    pub(crate) app_name: Option<&'static str>,
    pub(crate) nest_app: Option<NestApp>,
    pub(crate) logging: Option<LoggingConfig>,
    pub(crate) config_path: Option<PathBuf>,
    pub(crate) startup_options: Option<crate::startup::TauriStartupOptions>,
    pub(crate) modules: Vec<Box<dyn Module>>,
    #[cfg(feature = "async")]
    pub(crate) with_task_runtime: bool,
    #[cfg(feature = "runtime")]
    pub(crate) builder_extension: Option<BuilderExtensionFn>,
}

impl TauriApp {
    /// Creates a new Tauri application host.
    pub fn new(app_name: &'static str) -> Self {
        Self {
            app_name: Some(app_name),
            nest_app: None,
            logging: None,
            config_path: None,
            startup_options: None,
            modules: Vec::new(),
            #[cfg(feature = "async")]
            with_task_runtime: false,
            #[cfg(feature = "runtime")]
            builder_extension: None,
        }
    }

    /// Creates a Tauri host that executes a pre-built [`NestApp`] container.
    pub fn from_nest_app(nest_app: NestApp) -> Self {
        Self {
            app_name: None,
            nest_app: Some(nest_app),
            logging: None,
            config_path: None,
            startup_options: None,
            modules: Vec::new(),
            #[cfg(feature = "async")]
            with_task_runtime: false,
            #[cfg(feature = "runtime")]
            builder_extension: None,
        }
    }

    /// Attaches a pre-built [`NestApp`] container.
    pub fn with_nest_app(mut self, nest_app: NestApp) -> Self {
        self.nest_app = Some(nest_app);
        self.app_name = None;
        self
    }

    /// Sets pre-parsed startup options (otherwise parsed from args in `run`).
    pub fn startup_options(mut self, options: crate::startup::TauriStartupOptions) -> Self {
        self.startup_options = Some(options);
        self
    }

    /// Sets an explicit config path override (useful for tests).
    pub fn with_config_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.config_path = Some(path.into());
        self
    }

    /// Sets the base logging configuration before startup flags are applied.
    pub fn with_logging(mut self, config: LoggingConfig) -> Self {
        self.logging = Some(config);
        self
    }

    /// Registers a Nest module (ignored when a [`NestApp`] container is attached).
    pub fn module<M: Module + 'static>(mut self, module: M) -> Self {
        self.modules.push(Box::new(module));
        self
    }

    /// Registers the task runtime module when the `async` feature is enabled.
    #[cfg(feature = "async")]
    pub fn with_task_runtime(mut self, enabled: bool) -> Self {
        self.with_task_runtime = enabled;
        self
    }

    /// Extends the Tauri builder after the built-in IPC handler is attached.
    ///
    /// Use this to register application state (`manage`) and application IPC
    /// commands packaged as a Tauri plugin. Do **not** call `invoke_handler`
    /// inside the closure — that overwrites Nest's built-in commands. Plugin
    /// commands are invoked from the UI as `plugin:<name>|<command>`.
    ///
    /// ```ignore
    /// TauriApp::new("my-app")
    ///     .with_builder(|builder| {
    ///         builder
    ///             .manage(MyState::new())
    ///             .plugin(my_app::plugin())
    ///     })
    ///     .run(tauri::generate_context!());
    /// ```
    #[cfg(feature = "runtime")]
    pub fn with_builder<F>(mut self, extension: F) -> Self
    where
        F: FnOnce(tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> + Send + 'static,
    {
        self.builder_extension = Some(Box::new(extension));
        self
    }

    /// Runs the application using process arguments and a Tauri context from
    /// `tauri::generate_context!()` in the app `src-tauri` crate.
    #[cfg(feature = "runtime")]
    pub fn run(self, context: tauri::Context<tauri::Wry>) -> ! {
        match self.try_run(context) {
            Ok(()) => std::process::exit(0),
            Err(error) => exit_with_error(error),
        }
    }

    /// Runs the application and returns errors instead of exiting.
    #[cfg(feature = "runtime")]
    pub fn try_run(self, context: tauri::Context<tauri::Wry>) -> NestResult<()> {
        let args: Vec<OsString> = std::env::args_os().collect();
        self.try_run_with(context, args)
    }

    /// Runs the application with explicit arguments (useful for tests).
    #[cfg(feature = "runtime")]
    pub fn try_run_with<I, S>(self, context: tauri::Context<tauri::Wry>, args: I) -> NestResult<()>
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        let args: Vec<OsString> = args.into_iter().map(Into::into).collect();
        run_with_context(self, args, context)
    }

    /// Prepares runtime services without starting Tauri (tests).
    pub fn try_prepare_runtime<I, S>(self, args: I) -> NestResult<crate::bootstrap::PreparedRuntime>
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        let args: Vec<OsString> = args.into_iter().map(Into::into).collect();
        prepare_runtime(self, args)
    }
}
