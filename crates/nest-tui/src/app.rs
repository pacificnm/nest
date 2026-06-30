//! TUI application builder.

use std::ffi::OsString;
use std::path::PathBuf;

use nest_app::NestApp;
use nest_core::Module;
use nest_error::NestResult;
use nest_logging::LoggingConfig;

use crate::bootstrap::{exit_with_error, prepare_runtime, run_pipeline};
use crate::screen::TuiScreen;
use crate::startup::TuiStartupOptions;

/// Terminal UI host for Nest applications.
pub struct TuiApp {
    pub(crate) app_name: Option<&'static str>,
    pub(crate) nest_app: Option<NestApp>,
    pub(crate) logging: Option<LoggingConfig>,
    pub(crate) config_path: Option<PathBuf>,
    pub(crate) startup_options: Option<TuiStartupOptions>,
    pub(crate) modules: Vec<Box<dyn Module>>,
    pub(crate) screen: Option<Box<dyn TuiScreen>>,
    #[cfg(feature = "async")]
    pub(crate) with_task_runtime: bool,
}

impl TuiApp {
    /// Creates a new TUI application host.
    pub fn new(app_name: &'static str) -> Self {
        Self {
            app_name: Some(app_name),
            nest_app: None,
            logging: None,
            config_path: None,
            startup_options: None,
            modules: Vec::new(),
            screen: None,
            #[cfg(feature = "async")]
            with_task_runtime: false,
        }
    }

    /// Creates a TUI host that executes a pre-built [`NestApp`] container.
    pub fn from_nest_app(nest_app: NestApp) -> Self {
        Self {
            app_name: None,
            nest_app: Some(nest_app),
            logging: None,
            config_path: None,
            startup_options: None,
            modules: Vec::new(),
            screen: None,
            #[cfg(feature = "async")]
            with_task_runtime: false,
        }
    }

    /// Attaches a pre-built [`NestApp`] container.
    pub fn with_nest_app(mut self, nest_app: NestApp) -> Self {
        self.nest_app = Some(nest_app);
        self.app_name = None;
        self
    }

    /// Sets pre-parsed startup options (otherwise parsed from args in `run`).
    pub fn startup_options(mut self, options: TuiStartupOptions) -> Self {
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

    /// Registers the root TUI screen.
    pub fn screen<S: TuiScreen>(mut self, screen: S) -> Self {
        self.screen = Some(Box::new(screen));
        self
    }

    /// Registers the task runtime module when the `async` feature is enabled.
    #[cfg(feature = "async")]
    pub fn with_task_runtime(mut self, enabled: bool) -> Self {
        self.with_task_runtime = enabled;
        self
    }

    /// Runs the application using process arguments.
    pub fn run(self) -> ! {
        match self.try_run() {
            Ok(()) => std::process::exit(0),
            Err(error) => exit_with_error(error, None),
        }
    }

    /// Runs the application and returns errors instead of exiting.
    pub fn try_run(self) -> NestResult<()> {
        let args: Vec<OsString> = std::env::args_os().collect();
        self.try_run_with(args)
    }

    /// Runs the application with explicit arguments (useful for tests).
    pub fn try_run_with<I, S>(self, args: I) -> NestResult<()>
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        let args: Vec<OsString> = args.into_iter().map(Into::into).collect();
        run_pipeline(self, args)
    }

    /// Prepares runtime services without initializing the terminal (tests).
    pub fn try_prepare_runtime<I, S>(self, args: I) -> NestResult<crate::bootstrap::PreparedRuntime>
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        let args: Vec<OsString> = args.into_iter().map(Into::into).collect();
        prepare_runtime(self, args)
    }
}
