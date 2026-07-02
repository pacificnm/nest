//! Ordered TUI startup pipeline.

use std::ffi::OsString;

use nest_app::{AppBootstrapper, AppMetadata, NestApp};
use nest_config::{ConfigDocument, ConfigLoader, ConfigService};
use nest_core::{AppBuilder, AppContext, Module, ModuleId, NestResult};
use nest_error::NestError;
use nest_logging::LoggingConfig;

use crate::app::TuiApp;
use crate::config::{merge_runtime_config, TuiRuntimeConfig};
use crate::event_loop::run_event_loop;
use crate::logging::build_logging_config;
use crate::module::TuiModule;
use crate::render::render_nest_error;
use crate::startup::TuiStartupOptions;
use crate::terminal::{create_terminal, TerminalGuard};

#[cfg(feature = "async")]
use nest_task_runtime::{RuntimeConfig, TaskRuntimeModule};

/// Prepared runtime without terminal initialization (for tests).
pub struct PreparedRuntime {
    /// Nest application container.
    pub nest_app: NestApp,
    /// Merged TUI runtime settings.
    pub runtime_config: TuiRuntimeConfig,
    /// Parsed startup options.
    pub startup: TuiStartupOptions,
}

/// Runs the full TUI bootstrap pipeline including the event loop.
pub fn run_pipeline(mut app: TuiApp, args: Vec<OsString>) -> NestResult<()> {
    let mut screen = app
        .screen
        .take()
        .ok_or_else(|| NestError::command("TUI application requires a root screen"))?;

    let prepared = prepare_runtime(app, args)?;
    let PreparedRuntime {
        mut nest_app,
        runtime_config,
        ..
    } = prepared;

    let _guard = TerminalGuard::enter(&runtime_config)?;
    let terminal = create_terminal()?;
    let ctx = nest_app.context_arc();

    let result = run_event_loop(terminal, &runtime_config, screen.as_mut(), ctx.as_ref());
    nest_app.shutdown()?;
    result
}

/// Prepares config, logging, and `AppContext` without touching the terminal.
pub fn prepare_runtime(mut app: TuiApp, args: Vec<OsString>) -> NestResult<PreparedRuntime> {
    if let Some(nest_app) = app.nest_app.take() {
        return prepare_from_nest_app(app, nest_app, args);
    }

    let app_name = app
        .app_name
        .expect("TUI host requires app name or nest_app container");

    let startup = match app.startup_options {
        Some(options) => options,
        None => parse_startup_options(&args)?,
    };

    let config_path = app.config_path.or(startup.config_path.clone());
    let loaded = ConfigLoader::file_or_search(app_name, config_path).load()?;
    let document = loaded.document.clone();
    init_host_logging(app_name, &document, &startup, app.logging)?;

    let runtime_config = merge_runtime_config(&document, &startup)?;

    let mut builder = AppBuilder::new();
    builder.register_service(ConfigService::new(loaded))?;
    builder.register_service(startup.clone())?;
    builder.register_service(runtime_config.clone())?;

    let modules = app.modules;
    #[cfg(feature = "async")]
    if app.with_task_runtime && !has_runtime_module(&modules) {
        builder = builder.module(TaskRuntimeModule::owned(RuntimeConfig::default())?);
    }

    builder = builder.module(TuiModule);
    for module in modules {
        builder = builder.module(DynModule(module));
    }

    let mut nest_app = AppBootstrapper::build(AppMetadata::new(app_name), builder.build()?)?;
    nest_app.startup()?;

    Ok(PreparedRuntime {
        nest_app,
        runtime_config,
        startup,
    })
}

fn prepare_from_nest_app(
    app: TuiApp,
    mut nest_app: NestApp,
    args: Vec<OsString>,
) -> NestResult<PreparedRuntime> {
    let app_name = nest_app.metadata().name.clone();

    let startup = match app.startup_options {
        Some(options) => options,
        None => parse_startup_options(&args)?,
    };

    let config_path = app.config_path.or(startup.config_path.clone());
    let document = load_config_document(&app_name, config_path, Some(nest_app.context()))?;
    init_host_logging(&app_name, &document, &startup, app.logging)?;

    let runtime_config = merge_runtime_config(&document, &startup)?;
    nest_app.startup()?;

    Ok(PreparedRuntime {
        nest_app,
        runtime_config,
        startup,
    })
}

fn load_config_document(
    app_name: &str,
    config_path: Option<std::path::PathBuf>,
    ctx: Option<&AppContext>,
) -> NestResult<ConfigDocument> {
    if let Some(ctx) = ctx {
        if let Ok(config) = ctx.service::<ConfigService>() {
            return Ok(config.document().clone());
        }
    }

    let loaded = ConfigLoader::file_or_search(app_name, config_path).load()?;
    Ok(loaded.document)
}

fn init_host_logging(
    app_name: &str,
    document: &ConfigDocument,
    startup: &TuiStartupOptions,
    logging: Option<LoggingConfig>,
) -> NestResult<()> {
    let base_logging = logging.unwrap_or_else(|| LoggingConfig::for_tui(app_name));
    let logging_config = build_logging_config(base_logging, document, startup)?;
    if !tracing::dispatcher::has_been_set() {
        nest_logging::init(logging_config)?;
    }
    Ok(())
}

/// Handles fatal errors by rendering and exiting the process.
pub fn exit_with_error(error: NestError, runtime: Option<&TuiRuntimeConfig>) -> ! {
    let runtime = runtime.cloned().unwrap_or_default();
    render_nest_error(&error, &runtime);
    std::process::exit(1);
}

fn parse_startup_options(args: &[OsString]) -> NestResult<TuiStartupOptions> {
    let str_args: Vec<String> = args
        .iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();
    let arg_refs: Vec<&str> = str_args.iter().map(String::as_str).collect();
    let matches = crate::startup::build_startup_command()
        .try_get_matches_from(arg_refs)
        .map_err(|error| {
            NestError::command(error.to_string()).with_code(nest_error::codes::NEST_CLI_USAGE)
        })?;
    Ok(TuiStartupOptions::from_matches(&matches))
}

#[cfg(feature = "async")]
fn has_runtime_module(modules: &[Box<dyn Module>]) -> bool {
    use nest_task_runtime::TASK_RUNTIME_MODULE_ID;
    modules
        .iter()
        .any(|module| module.id() == TASK_RUNTIME_MODULE_ID)
}

struct DynModule(Box<dyn Module>);

impl Module for DynModule {
    fn id(&self) -> ModuleId {
        self.0.id()
    }

    fn dependencies(&self) -> &'static [ModuleId] {
        self.0.dependencies()
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        self.0.configure(app)
    }
}
