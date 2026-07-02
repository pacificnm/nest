//! Startup option parsing for TUI applications.

use std::ffi::OsString;
use std::path::PathBuf;

use clap::{Arg, ArgAction, ArgMatches, Command};
use nest_logging::LogLevel;

/// Parsed TUI startup flags (CLI overrides config).
#[derive(Debug, Clone, Default)]
pub struct TuiStartupOptions {
    /// Explicit `--config` path.
    pub config_path: Option<PathBuf>,
    /// `--log-level` override.
    pub log_level: Option<LogLevel>,
    /// `--log-file` path.
    pub log_file: Option<PathBuf>,
    /// `--no-color` disables ANSI styling in host error output.
    pub no_color: bool,
    /// Mouse support when set by `--mouse` or `--no-mouse`.
    pub mouse: Option<bool>,
    /// `--tick-rate` override in milliseconds.
    pub tick_rate_ms: Option<u64>,
    /// `--debug` enables debug logging.
    pub debug: bool,
}

impl TuiStartupOptions {
    /// Parses startup options from process arguments.
    pub fn from_args() -> nest_error::NestResult<Self> {
        let args: Vec<OsString> = std::env::args_os().collect();
        Self::from_args_iter(args)
    }

    /// Parses startup options from an explicit argument list.
    pub fn from_args_iter<I, S>(args: I) -> nest_error::NestResult<Self>
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        let args: Vec<OsString> = args.into_iter().map(Into::into).collect();
        let str_args: Vec<String> = args
            .iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        let arg_refs: Vec<&str> = str_args.iter().map(String::as_str).collect();
        let matches = build_startup_command()
            .try_get_matches_from(arg_refs)
            .map_err(|error| {
                nest_error::NestError::command(error.to_string())
                    .with_code(nest_error::codes::NEST_CLI_USAGE)
            })?;
        Ok(Self::from_matches(&matches))
    }

    /// Parses startup options from clap matches.
    pub fn from_matches(matches: &ArgMatches) -> Self {
        let log_level = matches
            .get_one::<String>("log-level")
            .and_then(|value| value.parse().ok());

        let mouse = if matches.get_flag("mouse") {
            Some(true)
        } else if matches.get_flag("no-mouse") {
            Some(false)
        } else {
            None
        };

        let tick_rate_ms = matches.get_one::<u64>("tick-rate").copied();

        Self {
            config_path: matches.get_one::<String>("config").map(PathBuf::from),
            log_level,
            log_file: matches.get_one::<String>("log-file").map(PathBuf::from),
            no_color: matches.get_flag("no-color"),
            mouse,
            tick_rate_ms,
            debug: matches.get_flag("debug"),
        }
    }
}

/// Builds the clap command used for startup option parsing.
pub fn build_startup_command() -> Command {
    Command::new("nest-tui")
        .arg(
            Arg::new("config")
                .long("config")
                .value_name("PATH")
                .help("Application configuration file"),
        )
        .arg(
            Arg::new("log-level")
                .long("log-level")
                .value_name("LEVEL")
                .help("Log level (trace, debug, info, warn, error)"),
        )
        .arg(
            Arg::new("log-file")
                .long("log-file")
                .value_name("PATH")
                .help("Write logs to the given file path"),
        )
        .arg(
            Arg::new("no-color")
                .long("no-color")
                .action(ArgAction::SetTrue)
                .help("Disable colored error output"),
        )
        .arg(
            Arg::new("mouse")
                .long("mouse")
                .action(ArgAction::SetTrue)
                .help("Enable mouse support")
                .conflicts_with("no-mouse"),
        )
        .arg(
            Arg::new("no-mouse")
                .long("no-mouse")
                .action(ArgAction::SetTrue)
                .help("Disable mouse support")
                .conflicts_with("mouse"),
        )
        .arg(
            Arg::new("tick-rate")
                .long("tick-rate")
                .value_name("MS")
                .value_parser(clap::value_parser!(u64))
                .help("UI redraw/event poll interval in milliseconds"),
        )
        .arg(
            Arg::new("debug")
                .long("debug")
                .action(ArgAction::SetTrue)
                .help("Enable debug logging"),
        )
}
