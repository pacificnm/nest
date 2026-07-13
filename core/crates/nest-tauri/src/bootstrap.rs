//! Ordered Tauri startup pipeline.

use std::ffi::OsString;

use nest_app::{AppBootstrapper, AppMetadata, NestApp};
use nest_config::{ConfigDocument, ConfigLoader, ConfigService};
use nest_core::{AppBuilder, AppContext, Module, ModuleId, NestResult};
use nest_error::NestError;
use nest_logging::LoggingConfig;

use crate::app::TauriApp;
use crate::config::{merge_runtime_config, TauriRuntimeConfig};
use crate::logging::build_logging_config;
use crate::module::TauriModule;
use crate::startup::TauriStartupOptions;

#[cfg(feature = "runtime")]
use nest_error::codes::NEST_TAURI_START_FAILED;

#[cfg(feature = "runtime")]
use crate::commands::attach_invoke_handler;

#[cfg(feature = "runtime")]
use crate::state::NestHostState;

#[cfg(feature = "runtime")]
use tauri::{Manager, RunEvent, WebviewUrl, WebviewWindowBuilder};

#[cfg(feature = "async")]
use nest_task_runtime::{RuntimeConfig, TaskRuntimeModule};

/// Prepared runtime without starting Tauri (for tests).
pub struct PreparedRuntime {
    /// Nest application container.
    pub nest_app: NestApp,
    /// Merged Tauri runtime settings.
    pub runtime_config: TauriRuntimeConfig,
    /// Parsed startup options.
    pub startup: TauriStartupOptions,
}

/// Runs the full Tauri bootstrap pipeline including the webview event loop.
#[cfg(feature = "runtime")]
pub fn run_with_context(
    mut app: TauriApp,
    args: Vec<OsString>,
    context: tauri::Context<tauri::Wry>,
) -> NestResult<()> {
    let builder_extension = app.builder_extension.take();
    let prepared = prepare_runtime(app, args)?;
    let runtime_config = prepared.runtime_config.clone();
    let host_state = NestHostState::new(prepared.nest_app, prepared.runtime_config);

    let builder = attach_invoke_handler(tauri::Builder::default().manage(host_state));
    let builder = match builder_extension {
        Some(extend) => extend(builder),
        None => builder,
    };

    let tauri_app = builder
        .setup(move |tauri_app| {
            apply_window_settings(tauri_app, &runtime_config)?;
            Ok(())
        })
        .build(context)
        .map_err(|error| {
            NestError::ui(format!("tauri build failed: {error}")).with_code(NEST_TAURI_START_FAILED)
        })?;

    tauri_app.run(|app_handle, event| {
        if matches!(event, RunEvent::Exit) {
            if let Some(state) = app_handle.try_state::<NestHostState>() {
                let _ = state.shutdown();
            }
        }
    });

    Ok(())
}

/// Prepares config, logging, and `AppContext` without starting Tauri.
pub fn prepare_runtime(mut app: TauriApp, args: Vec<OsString>) -> NestResult<PreparedRuntime> {
    if let Some(nest_app) = app.nest_app.take() {
        return prepare_from_nest_app(app, nest_app, args);
    }

    let app_name = app
        .app_name
        .expect("Tauri host requires app name or nest_app container");

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

    builder = builder.module(TauriModule);
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
    app: TauriApp,
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
    startup: &TauriStartupOptions,
    logging: Option<LoggingConfig>,
) -> NestResult<()> {
    let base_logging = logging.unwrap_or_else(|| LoggingConfig::for_tauri(app_name));
    let logging_config = build_logging_config(base_logging, document, startup)?;
    if !tracing::dispatcher::has_been_set() {
        nest_logging::init(logging_config)?;
    }
    Ok(())
}

fn parse_startup_options(args: &[OsString]) -> NestResult<TauriStartupOptions> {
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
    Ok(TauriStartupOptions::from_matches(&matches))
}

#[cfg(feature = "runtime")]
fn apply_window_settings(app: &tauri::App, runtime_config: &TauriRuntimeConfig) -> NestResult<()> {
    if let Some(window) = app.get_webview_window("main") {
        window
            .set_title(&runtime_config.title)
            .map_err(|error| NestError::ui(format!("set title failed: {error}")))?;
        window
            .set_size(tauri::Size::Logical(tauri::LogicalSize {
                width: runtime_config.width as f64,
                height: runtime_config.height as f64,
            }))
            .map_err(|error| NestError::ui(format!("set size failed: {error}")))?;
        return Ok(());
    }

    WebviewWindowBuilder::new(app, "main", WebviewUrl::default())
        .title(&runtime_config.title)
        .inner_size(runtime_config.width as f64, runtime_config.height as f64)
        .build()
        .map_err(|error| NestError::ui(format!("create window failed: {error}")))?;

    Ok(())
}

/// Handles fatal errors by printing and exiting the process.
#[cfg(feature = "runtime")]
pub fn exit_with_error(error: NestError) -> ! {
    eprintln!("{error}");
    std::process::exit(1);
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
