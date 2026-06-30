//! Ordered GUI startup pipeline.

use std::ffi::OsString;

use nest_app::{AppBootstrapper, AppMetadata, NestApp};
use nest_config::{ConfigDocument, ConfigLoader, ConfigService};
use nest_core::{AppBuilder, AppContext, Module, ModuleId, NestResult};
use nest_error::NestError;
use nest_logging::LoggingConfig;

use crate::app::GuiApp;
use crate::config::{merge_runtime_config, GuiRuntimeConfig};
use crate::logging::build_logging_config;
use crate::module::GuiModule;
use crate::render::render_nest_error;
use crate::shell::run_eframe;
use crate::startup::GuiStartupOptions;

#[cfg(feature = "async")]
use nest_task_runtime::{RuntimeConfig, TaskRuntimeModule};

/// Prepared runtime without eframe initialization (for tests).
pub struct PreparedRuntime {
    /// Nest application container.
    pub nest_app: NestApp,
    /// Merged GUI runtime settings.
    pub runtime_config: GuiRuntimeConfig,
    /// Parsed startup options.
    pub startup: GuiStartupOptions,
}

/// Runs the full GUI bootstrap pipeline including the eframe loop.
pub fn run_pipeline(mut app: GuiApp, args: Vec<OsString>) -> NestResult<()> {
    let view = app
        .view
        .take()
        .ok_or_else(|| NestError::command("GUI application requires a root view"))?;

    let prepared = prepare_runtime(app, args)?;
    let PreparedRuntime {
        mut nest_app,
        runtime_config,
        ..
    } = prepared;

    let ctx = nest_app.context_arc();
    let result = run_eframe(&runtime_config, ctx, view);
    nest_app.shutdown()?;
    result
}

/// Prepares config, logging, and `AppContext` without starting eframe.
pub fn prepare_runtime(mut app: GuiApp, args: Vec<OsString>) -> NestResult<PreparedRuntime> {
    if let Some(nest_app) = app.nest_app.take() {
        return prepare_from_nest_app(app, nest_app, args);
    }

    let app_name = app
        .app_name
        .expect("GUI host requires app name or nest_app container");

    let startup = match app.startup_options {
        Some(options) => options,
        None => parse_startup_options(&args)?,
    };

    let config_path = app.config_path.or(startup.config_path.clone());
    let loaded = ConfigLoader::file_or_search(app_name, config_path).load()?;
    let document = loaded.document.clone();
    init_host_logging(app_name, &document, &startup, app.logging)?;

    let runtime_config = merge_runtime_config(app_name, &document, &startup)?;

    let mut builder = AppBuilder::new();
    builder.register_service(ConfigService::new(loaded))?;
    builder.register_service(startup.clone())?;
    builder.register_service(runtime_config.clone())?;

    let modules = app.modules;
    #[cfg(feature = "async")]
    if app.with_task_runtime && !has_runtime_module(&modules) {
        builder = builder.module(TaskRuntimeModule::owned(RuntimeConfig::default())?);
    }

    builder = builder.module(GuiModule);
    for module in modules {
        builder = builder.module(DynModule(module));
    }

    let mut nest_app =
        AppBootstrapper::build(AppMetadata::new(app_name), builder.build()?)?;
    nest_app.startup()?;

    Ok(PreparedRuntime {
        nest_app,
        runtime_config,
        startup,
    })
}

fn prepare_from_nest_app(
    app: GuiApp,
    mut nest_app: NestApp,
    args: Vec<OsString>,
) -> NestResult<PreparedRuntime> {
    let app_name = nest_app.metadata().name.clone();

    let startup = match app.startup_options {
        Some(options) => options,
        None => parse_startup_options(&args)?,
    };

    let config_path = app.config_path.or(startup.config_path.clone());
    let document = load_config_document(
        &app_name,
        config_path,
        Some(nest_app.context()),
    )?;
    init_host_logging(&app_name, &document, &startup, app.logging)?;

    let runtime_config = merge_runtime_config(&app_name, &document, &startup)?;
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
    startup: &GuiStartupOptions,
    logging: Option<LoggingConfig>,
) -> NestResult<()> {
    let base_logging = logging.unwrap_or_else(|| LoggingConfig::for_gui(app_name));
    let logging_config = build_logging_config(base_logging, document, startup)?;
    if !tracing::dispatcher::has_been_set() {
        nest_logging::init(logging_config)?;
    }
    Ok(())
}

/// Handles fatal errors by rendering and exiting the process.
pub fn exit_with_error(error: NestError, runtime: Option<&GuiRuntimeConfig>) -> ! {
    let runtime = runtime
        .cloned()
        .unwrap_or_else(|| GuiRuntimeConfig::with_app_name("nest-gui"));
    render_nest_error(&error, &runtime);
    std::process::exit(1);
}

fn parse_startup_options(args: &[OsString]) -> NestResult<GuiStartupOptions> {
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
    Ok(GuiStartupOptions::from_matches(&matches))
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
