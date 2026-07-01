//! CLI application builder.

use std::ffi::OsString;

use nest_app::NestApp;
use nest_core::Module;
use nest_error::NestResult;
use nest_logging::LoggingConfig;

use crate::bootstrap::{exit_with_error, run_pipeline};
use crate::command::CliCommand;
use crate::registry::CommandRegistry;

#[cfg(feature = "async")]
use crate::command::AsyncCliCommand;

/// Command-line host for Nest applications.
pub struct CliApp {
    pub(crate) app_name: Option<&'static str>,
    pub(crate) about: Option<&'static str>,
    pub(crate) long_about: Option<&'static str>,
    pub(crate) version: Option<&'static str>,
    pub(crate) nest_app: Option<NestApp>,
    pub(crate) logging: Option<LoggingConfig>,
    pub(crate) log_level_from_args: bool,
    pub(crate) modules: Vec<Box<dyn Module>>,
    pub(crate) registry: CommandRegistry,
}

impl CliApp {
    /// Creates a new CLI application host.
    pub fn new(app_name: &'static str) -> Self {
        Self {
            app_name: Some(app_name),
            about: None,
            long_about: None,
            version: None,
            nest_app: None,
            logging: None,
            log_level_from_args: false,
            modules: Vec::new(),
            registry: CommandRegistry::new(),
        }
    }

    /// Creates a CLI host that executes a pre-built [`NestApp`] container.
    pub fn from_nest_app(nest_app: NestApp) -> Self {
        Self {
            app_name: None,
            about: None,
            long_about: None,
            version: None,
            nest_app: Some(nest_app),
            logging: None,
            log_level_from_args: false,
            modules: Vec::new(),
            registry: CommandRegistry::new(),
        }
    }

    /// Attaches a pre-built [`NestApp`] container.
    pub fn with_nest_app(mut self, nest_app: NestApp) -> Self {
        self.nest_app = Some(nest_app);
        self.app_name = None;
        self
    }

    /// Sets the base logging configuration before CLI flags are applied.
    pub fn with_logging(mut self, config: LoggingConfig) -> Self {
        self.logging = Some(config);
        self
    }

    /// When true, `--log-level` overrides config file and defaults.
    pub fn with_log_level_from_args(mut self, enabled: bool) -> Self {
        self.log_level_from_args = enabled;
        self
    }

    /// Sets the application version used for `--version`, lifecycle logs, and host metadata.
    pub fn with_version(mut self, version: &'static str) -> Self {
        self.version = Some(version);
        self
    }

    /// Sets the short description shown in help output.
    pub fn with_about(mut self, about: &'static str) -> Self {
        self.about = Some(about);
        self
    }

    /// Sets the long description shown in help output (supports multiple lines).
    pub fn with_long_about(mut self, long_about: &'static str) -> Self {
        self.long_about = Some(long_about);
        self
    }

    /// Registers a Nest module (ignored when a [`NestApp`] container is attached).
    pub fn module<M: Module + 'static>(mut self, module: M) -> Self {
        self.modules.push(Box::new(module));
        self
    }

    /// Registers a synchronous CLI subcommand.
    pub fn command<C: CliCommand>(mut self, command: C) -> Self {
        self.registry.register_sync(Box::new(command));
        self
    }

    /// Registers an asynchronous CLI subcommand.
    #[cfg(feature = "async")]
    pub fn async_command<C: AsyncCliCommand>(mut self, command: C) -> Self {
        self.registry.register_async(Box::new(command));
        self
    }

    /// Runs the application using process arguments.
    pub fn run(self) -> ! {
        let args: Vec<OsString> = std::env::args_os().collect();
        match self.try_run_with(args) {
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

    /// Renders the long `--help` text for this application.
    pub fn render_long_help(&self) -> NestResult<String> {
        let name = self
            .app_name
            .ok_or_else(|| nest_error::NestError::command("CLI host requires an application name"))?;
        Ok(self
            .registry
            .build_clap_command(name, self.about, self.long_about, self.version)
            .render_long_help()
            .to_string())
    }

    /// Renders the long `--help` text for a top-level command group.
    pub fn render_group_long_help(&self, group: &str) -> NestResult<String> {
        let name = self
            .app_name
            .ok_or_else(|| nest_error::NestError::command("CLI host requires an application name"))?;
        let mut root = self
            .registry
            .build_clap_command(name, self.about, self.long_about, self.version);
        let sub = root.find_subcommand_mut(group).ok_or_else(|| {
            nest_error::NestError::command(format!("unknown command group: {group}"))
        })?;
        Ok(sub.render_long_help().to_string())
    }
}
