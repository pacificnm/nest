//! Parsed global CLI flags.

use std::path::PathBuf;

use clap::ArgMatches;
use nest_logging::LogLevel;

/// Global CLI options parsed before command dispatch.
#[derive(Debug, Clone)]
pub struct CliGlobals {
    /// Explicit `--config` path.
    pub config_path: Option<PathBuf>,
    /// `--log-level` override.
    pub log_level: Option<LogLevel>,
    /// `--log-file` path.
    pub log_file: Option<PathBuf>,
    /// `--json` output mode.
    pub json: bool,
    /// `--quiet` suppresses non-error stdout.
    pub quiet: bool,
    /// `--verbose` enables debug logging.
    pub verbose: bool,
    /// `--no-color` disables ANSI styling.
    pub no_color: bool,
}

impl CliGlobals {
    /// Parses global flags from clap matches.
    pub fn from_matches(matches: &ArgMatches) -> Self {
        let log_level = matches
            .get_one::<String>("log-level")
            .and_then(|value| value.parse().ok());

        Self {
            config_path: matches.get_one::<String>("config").map(PathBuf::from),
            log_level,
            log_file: matches.get_one::<String>("log-file").map(PathBuf::from),
            json: matches.get_flag("json"),
            quiet: matches.get_flag("quiet"),
            verbose: matches.get_flag("verbose"),
            no_color: matches.get_flag("no-color"),
        }
    }
}

/// clap argument ids for global flags.
pub mod arg {
    pub const CONFIG: &str = "config";
    pub const LOG_LEVEL: &str = "log-level";
    pub const LOG_FILE: &str = "log-file";
    pub const JSON: &str = "json";
    pub const QUIET: &str = "quiet";
    pub const VERBOSE: &str = "verbose";
    pub const NO_COLOR: &str = "no-color";
}

/// Attaches global arguments to a clap root command.
pub fn attach_global_args(cmd: clap::Command) -> clap::Command {
    use clap::Arg;
    use clap::ArgAction;

    cmd.arg(
        Arg::new(arg::CONFIG)
            .long("config")
            .value_name("PATH")
            .help("Application configuration file"),
    )
    .arg(
        Arg::new(arg::LOG_LEVEL)
            .long("log-level")
            .value_name("LEVEL")
            .help("Log level (trace, debug, info, warn, error)"),
    )
    .arg(
        Arg::new(arg::LOG_FILE)
            .long("log-file")
            .value_name("PATH")
            .help("Write logs to the given file path"),
    )
    .arg(
        Arg::new(arg::JSON)
            .long("json")
            .action(ArgAction::SetTrue)
            .help("Emit JSON output (errors and supported commands such as config show)"),
    )
    .arg(
        Arg::new(arg::QUIET)
            .long("quiet")
            .action(ArgAction::SetTrue)
            .help("Suppress non-essential output (warnings, human-readable summaries); JSON output is still emitted when --json is set"),
    )
    .arg(
        Arg::new(arg::VERBOSE)
            .long("verbose")
            .action(ArgAction::SetTrue)
            .help("Enable verbose output and debug logging"),
    )
    .arg(
        Arg::new(arg::NO_COLOR)
            .long("no-color")
            .action(ArgAction::SetTrue)
            .help("Disable colored error output"),
    )
}
