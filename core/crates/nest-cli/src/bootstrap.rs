//! Ordered CLI startup pipeline.

use std::ffi::OsString;

use nest_app::{AppBootstrapper, AppMetadata};
use nest_config::{ConfigDocument, ConfigLoader, ConfigService};
use nest_core::{AppBuilder, AppContext, Module, ModuleId, NestResult};
use nest_error::NestError;
use nest_logging::LoggingConfig;

use crate::app::CliApp;
use crate::codes::NEST_CLI_USAGE;
use crate::exit::CliExitCode;
use crate::globals::CliGlobals;
use crate::logging::build_logging_config;
use crate::module::CliModule;
use crate::render::render_error;

#[cfg(feature = "async")]
use nest_task_runtime::{RuntimeConfig, TaskRuntime, TaskRuntimeModule, TASK_RUNTIME_MODULE_ID};

/// Runs the full CLI bootstrap pipeline.
pub fn run_pipeline(mut app: CliApp, args: Vec<OsString>) -> NestResult<()> {
    let static_name = cli_app_name(&app);
    let app_name = static_name.to_string();
    let about = app.about;
    let long_about = app.long_about;
    let registry = app.registry;
    let clap_cmd = registry.build_clap_command(static_name, about, long_about);

    let str_args: Vec<String> = args
        .iter()
        .map(|arg| arg.to_string_lossy().into_owned())
        .collect();
    let arg_refs: Vec<&str> = str_args.iter().map(String::as_str).collect();

    let matches = match clap_cmd.try_get_matches_from(arg_refs) {
        Ok(matches) => matches,
        Err(error) if is_clap_help_or_version(&error) => {
            error.print().expect("failed to write clap help or version");
            return Ok(());
        }
        Err(error) => return Err(clap_to_nest_error(error)),
    };

    let globals = CliGlobals::from_matches(&matches);

    if let Some(mut nest_app) = app.nest_app.take() {
        let document = load_config_document(
            &app_name,
            globals.config_path.clone(),
            Some(nest_app.context()),
        )?;
        init_host_logging(
            &app_name,
            &document,
            &globals,
            app.logging,
            app.log_level_from_args,
        )?;

        nest_app.startup()?;
        let result = dispatch_command(&registry, nest_app.context(), &matches);
        nest_app.shutdown()?;
        return result;
    }

    let loaded = ConfigLoader::file_or_search(&app_name, globals.config_path.clone()).load()?;
    let document = loaded.document.clone();

    init_host_logging(
        &app_name,
        &document,
        &globals,
        app.logging,
        app.log_level_from_args,
    )?;

    let mut builder = AppBuilder::new();
    builder.register_service(ConfigService::new(loaded))?;
    builder.register_service(globals.clone())?;

    let modules = app.modules;
    #[cfg(feature = "async")]
    if registry.has_async_commands() && !has_runtime_module(&modules) {
        builder = builder.module(TaskRuntimeModule::owned(RuntimeConfig::default())?);
    }

    builder = builder.module(CliModule);
    for module in modules {
        builder = builder.module(DynModule(module));
    }

    let mut nest_app =
        AppBootstrapper::build(AppMetadata::new(&app_name), builder.build()?)?;
    nest_app.startup()?;

    let result = dispatch_command(&registry, nest_app.context(), &matches);

    nest_app.shutdown()?;
    result
}

/// Handles fatal errors by rendering and exiting the process.
pub fn exit_with_error(error: NestError, globals: Option<&CliGlobals>) -> ! {
    let globals = globals.cloned().unwrap_or_else(default_globals);
    render_error(&error, &globals);
    std::process::exit(CliExitCode::from_error(&error).as_i32());
}

fn cli_app_name(app: &CliApp) -> &'static str {
    if let Some(name) = app.app_name {
        return name;
    }

    let name = app
        .nest_app
        .as_ref()
        .expect("CLI host requires app name or nest_app container")
        .metadata()
        .name
        .clone();
    Box::leak(name.into_boxed_str())
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
    globals: &CliGlobals,
    logging: Option<LoggingConfig>,
    log_level_from_args: bool,
) -> NestResult<()> {
    let base_logging = logging.unwrap_or_else(|| LoggingConfig::for_cli(app_name));
    let logging_config =
        build_logging_config(base_logging, document, globals, log_level_from_args)?;
    if !tracing::dispatcher::has_been_set() {
        nest_logging::init(logging_config)?;
    }
    Ok(())
}

fn dispatch_command(
    registry: &crate::registry::CommandRegistry,
    ctx: &AppContext,
    matches: &clap::ArgMatches,
) -> NestResult<()> {
    let (name, sub_matches) = matches
        .subcommand()
        .ok_or_else(|| NestError::command("missing subcommand").with_code(NEST_CLI_USAGE))?;

    if let Some(command) = registry.find_sync(name) {
        return command.run(ctx, sub_matches);
    }

    #[cfg(feature = "async")]
    if let Some(command) = registry.find_async(name) {
        let runtime = ctx.service::<TaskRuntime>()?;
        return runtime
            .handle()
            .block_on(command.run_async(ctx, sub_matches));
    }

    #[cfg(not(feature = "async"))]
    let _ = name;

    Err(
        NestError::command(format!("unknown subcommand: {name}"))
            .with_code(NEST_CLI_USAGE),
    )
}

fn is_clap_help_or_version(error: &clap::Error) -> bool {
    matches!(
        error.kind(),
        clap::error::ErrorKind::DisplayHelp
            | clap::error::ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand
            | clap::error::ErrorKind::DisplayVersion
    )
}

fn clap_to_nest_error(error: clap::Error) -> NestError {
    NestError::command(error.to_string()).with_code(NEST_CLI_USAGE)
}

fn default_globals() -> CliGlobals {
    CliGlobals {
        config_path: None,
        log_level: None,
        log_file: None,
        json: false,
        quiet: false,
        verbose: false,
        no_color: false,
    }
}

#[cfg(feature = "async")]
fn has_runtime_module(modules: &[Box<dyn Module>]) -> bool {
    modules.iter().any(|module| module.id() == TASK_RUNTIME_MODULE_ID)
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
