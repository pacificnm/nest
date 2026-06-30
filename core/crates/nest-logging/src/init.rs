//! Subscriber initialization and guard management.

use std::fs;
use std::io;
use std::sync::OnceLock;

use nest_error::{NestError, NestResult};
use tracing_appender::non_blocking::{NonBlocking, WorkerGuard};
use tracing_subscriber::layer::Layered;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

use crate::codes::{
    NEST_LOGGING_ALREADY_INIT, NEST_LOGGING_DIR_CREATE, NEST_LOGGING_NO_DIRECTORY,
    NEST_LOGGING_NO_TARGETS,
};
use crate::config::LoggingConfig;
use crate::filter::build_env_filter;
use crate::format::LogFormat;
use crate::panic_hook::install_panic_hook;
use crate::retention::cleanup_logs;
use crate::rotation::{file_appender, validate_rotation};

static GLOBAL_GUARD: OnceLock<LoggingGuard> = OnceLock::new();

type FilteredRegistry = Layered<EnvFilter, Registry>;

/// Holds non-blocking writer guards so file logging continues until dropped.
#[derive(Debug)]
pub struct LoggingGuard {
    _guards: Vec<WorkerGuard>,
}

impl LoggingGuard {
    fn new(guards: Vec<WorkerGuard>) -> Self {
        Self { _guards: guards }
    }
}

/// Initializes global logging and stores the guard internally for process lifetime.
pub fn init(config: LoggingConfig) -> NestResult<()> {
    let guard = init_logging(config)?;
    GLOBAL_GUARD.set(guard).map_err(|_| {
        NestError::config("logging guard already stored").with_code(NEST_LOGGING_ALREADY_INIT)
    })
}

/// Initializes global logging and returns a guard that must be kept alive.
pub fn init_logging(config: LoggingConfig) -> NestResult<LoggingGuard> {
    if tracing::dispatcher::has_been_set() {
        return Err(NestError::config("tracing subscriber already initialized")
            .with_code(NEST_LOGGING_ALREADY_INIT));
    }

    validate_config(&config)?;

    let rotation = validate_rotation(config.rotation)?;
    let env_filter = build_env_filter(&config)?;

    if let Some(directory) = config.directory.as_deref() {
        fs::create_dir_all(directory).map_err(|err| {
            NestError::io(format!("failed to create log directory: {directory:?}"))
                .with_code(NEST_LOGGING_DIR_CREATE)
                .with_source(err)
        })?;
        let _ = cleanup_logs(directory, &config.app_name, config.retention)?;
    }

    let mut guards = Vec::new();
    let registry = Registry::default().with(env_filter);

    match (
        config.has_console(),
        config.has_file(),
        config.has_json_file(),
    ) {
        (true, false, false) => init_console_only(registry, config.format),
        (false, true, false) => {
            let (writer, guard) = text_file_writer(&config, &rotation)?;
            guards.push(guard);
            init_file_only(registry, writer, config.format);
        }
        (false, false, true) => {
            let (writer, guard) = json_file_writer(&config, &rotation)?;
            guards.push(guard);
            init_json_file_only(registry, writer);
        }
        (true, true, false) => {
            let (writer, guard) = text_file_writer(&config, &rotation)?;
            guards.push(guard);
            init_console_and_file(registry, writer, config.format);
        }
        (true, false, true) => {
            let (writer, guard) = json_file_writer(&config, &rotation)?;
            guards.push(guard);
            init_console_and_json(registry, writer, config.format);
        }
        (false, true, true) => {
            let (text, guard1) = text_file_writer(&config, &rotation)?;
            let (json, guard2) = json_file_writer(&config, &rotation)?;
            guards.push(guard1);
            guards.push(guard2);
            init_text_and_json(registry, text, json);
        }
        (true, true, true) => {
            let (text, guard1) = text_file_writer(&config, &rotation)?;
            let (json, guard2) = json_file_writer(&config, &rotation)?;
            guards.push(guard1);
            guards.push(guard2);
            init_console_text_and_json(registry, text, json, config.format);
        }
        (false, false, false) => {
            return Err(
                NestError::config("at least one log target must be configured")
                    .with_code(NEST_LOGGING_NO_TARGETS),
            );
        }
    }

    if config.capture_panics {
        install_panic_hook();
    }

    Ok(LoggingGuard::new(guards))
}

fn validate_config(config: &LoggingConfig) -> NestResult<()> {
    if config.targets.is_empty() {
        return Err(
            NestError::config("at least one log target must be configured")
                .with_code(NEST_LOGGING_NO_TARGETS),
        );
    }

    if (config.has_file() || config.has_json_file()) && config.directory.is_none() {
        return Err(
            NestError::config("log directory is required for file targets")
                .with_code(NEST_LOGGING_NO_DIRECTORY),
        );
    }

    Ok(())
}

fn text_file_writer(
    config: &LoggingConfig,
    rotation: &tracing_appender::rolling::Rotation,
) -> NestResult<(NonBlocking, WorkerGuard)> {
    let directory = config.directory.as_deref().ok_or_else(|| {
        NestError::config("log directory is required").with_code(NEST_LOGGING_NO_DIRECTORY)
    })?;
    let appender = file_appender(directory, &config.app_name, rotation.clone());
    Ok(tracing_appender::non_blocking(appender))
}

fn json_file_writer(
    config: &LoggingConfig,
    rotation: &tracing_appender::rolling::Rotation,
) -> NestResult<(NonBlocking, WorkerGuard)> {
    let directory = config.directory.as_deref().ok_or_else(|| {
        NestError::config("log directory is required").with_code(NEST_LOGGING_NO_DIRECTORY)
    })?;
    let json_name = format!("{}-json", config.app_name);
    let appender = file_appender(directory, &json_name, rotation.clone());
    Ok(tracing_appender::non_blocking(appender))
}

fn init_console_only(registry: FilteredRegistry, format: LogFormat) {
    match format {
        LogFormat::Pretty => registry
            .with(
                fmt::layer()
                    .pretty()
                    .with_writer(io::stdout)
                    .with_ansi(true),
            )
            .init(),
        LogFormat::Compact => registry
            .with(
                fmt::layer()
                    .compact()
                    .with_writer(io::stdout)
                    .with_ansi(true),
            )
            .init(),
        LogFormat::Json => registry
            .with(fmt::layer().json().with_writer(io::stdout).with_ansi(false))
            .init(),
    }
}

fn init_file_only(registry: FilteredRegistry, writer: NonBlocking, format: LogFormat) {
    match format {
        LogFormat::Pretty => registry
            .with(fmt::layer().pretty().with_writer(writer).with_ansi(false))
            .init(),
        LogFormat::Compact => registry
            .with(fmt::layer().compact().with_writer(writer).with_ansi(false))
            .init(),
        LogFormat::Json => registry
            .with(fmt::layer().json().with_writer(writer).with_ansi(false))
            .init(),
    }
}

fn init_json_file_only(registry: FilteredRegistry, writer: NonBlocking) {
    registry
        .with(fmt::layer().json().with_writer(writer).with_ansi(false))
        .init();
}

fn init_console_and_file(registry: FilteredRegistry, writer: NonBlocking, format: LogFormat) {
    match format {
        LogFormat::Pretty => registry
            .with(fmt::layer().pretty().with_writer(writer).with_ansi(false))
            .with(
                fmt::layer()
                    .pretty()
                    .with_writer(io::stdout)
                    .with_ansi(true),
            )
            .init(),
        LogFormat::Compact => registry
            .with(fmt::layer().compact().with_writer(writer).with_ansi(false))
            .with(
                fmt::layer()
                    .compact()
                    .with_writer(io::stdout)
                    .with_ansi(true),
            )
            .init(),
        LogFormat::Json => registry
            .with(fmt::layer().json().with_writer(writer).with_ansi(false))
            .with(fmt::layer().json().with_writer(io::stdout).with_ansi(false))
            .init(),
    }
}

fn init_console_and_json(registry: FilteredRegistry, writer: NonBlocking, format: LogFormat) {
    match format {
        LogFormat::Pretty => registry
            .with(fmt::layer().json().with_writer(writer).with_ansi(false))
            .with(
                fmt::layer()
                    .pretty()
                    .with_writer(io::stdout)
                    .with_ansi(true),
            )
            .init(),
        LogFormat::Compact => registry
            .with(fmt::layer().json().with_writer(writer).with_ansi(false))
            .with(
                fmt::layer()
                    .compact()
                    .with_writer(io::stdout)
                    .with_ansi(true),
            )
            .init(),
        LogFormat::Json => registry
            .with(fmt::layer().json().with_writer(writer).with_ansi(false))
            .with(fmt::layer().json().with_writer(io::stdout).with_ansi(false))
            .init(),
    }
}

fn init_text_and_json(registry: FilteredRegistry, text: NonBlocking, json: NonBlocking) {
    registry
        .with(fmt::layer().pretty().with_writer(text).with_ansi(false))
        .with(fmt::layer().json().with_writer(json).with_ansi(false))
        .init();
}

fn init_console_text_and_json(
    registry: FilteredRegistry,
    text: NonBlocking,
    json: NonBlocking,
    format: LogFormat,
) {
    match format {
        LogFormat::Pretty => registry
            .with(fmt::layer().pretty().with_writer(text).with_ansi(false))
            .with(fmt::layer().json().with_writer(json).with_ansi(false))
            .with(
                fmt::layer()
                    .pretty()
                    .with_writer(io::stdout)
                    .with_ansi(true),
            )
            .init(),
        LogFormat::Compact => registry
            .with(fmt::layer().compact().with_writer(text).with_ansi(false))
            .with(fmt::layer().json().with_writer(json).with_ansi(false))
            .with(
                fmt::layer()
                    .compact()
                    .with_writer(io::stdout)
                    .with_ansi(true),
            )
            .init(),
        LogFormat::Json => registry
            .with(fmt::layer().json().with_writer(text).with_ansi(false))
            .with(fmt::layer().json().with_writer(json).with_ansi(false))
            .with(fmt::layer().json().with_writer(io::stdout).with_ansi(false))
            .init(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::level::LogLevel;
    use crate::target::LogTarget;

    #[test]
    fn init_requires_targets() {
        let err = validate_config(&LoggingConfig::new("kiwi")).unwrap_err();
        assert_eq!(err.code(), Some(NEST_LOGGING_NO_TARGETS));
    }

    #[test]
    fn file_requires_directory() {
        let mut config = LoggingConfig::new("kiwi");
        config.targets.push(LogTarget::File);
        let err = validate_config(&config).unwrap_err();
        assert_eq!(err.code(), Some(NEST_LOGGING_NO_DIRECTORY));
    }

    #[test]
    fn config_accepts_console_target() {
        let config = LoggingConfig::new("kiwi")
            .with_console()
            .with_default_level(LogLevel::Info);
        assert!(config.has_console());
        validate_config(&config).unwrap();
    }
}
