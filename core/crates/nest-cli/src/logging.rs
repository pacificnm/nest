//! Merge logging defaults, config section, and CLI flags.

use nest_error::NestResult;
use nest_logging::{LogLevel, LoggingConfig};
use serde::Deserialize;

use crate::globals::CliGlobals;
use nest_config::ConfigDocument;

#[derive(Debug, Deserialize)]
struct LoggingSection {
    level: Option<String>,
    directory: Option<String>,
}

/// Builds the final logging configuration for CLI startup.
pub fn build_logging_config(
    base: LoggingConfig,
    document: &ConfigDocument,
    globals: &CliGlobals,
    log_level_from_args: bool,
) -> NestResult<LoggingConfig> {
    let mut config = base;

    if document.has_section("logging") {
        let section: LoggingSection = document.section("logging")?;
        if let Some(level) = section.level.as_deref() {
            if let Ok(parsed) = level.parse::<LogLevel>() {
                config.level = parsed;
            }
        }
        if let Some(directory) = section.directory {
            config = config.with_file(directory);
        }
    }

    if globals.verbose {
        config.level = LogLevel::Debug;
    }

    if log_level_from_args {
        if let Some(level) = globals.log_level {
            config.level = level;
        }
    }

    if globals.quiet && !globals.verbose && globals.log_level.is_none() {
        config.level = LogLevel::Warn;
    }

    if let Some(log_file) = &globals.log_file {
        let parent = log_file
            .parent()
            .filter(|path| !path.as_os_str().is_empty());
        if let Some(directory) = parent {
            config = config.with_file(directory);
        } else {
            config = config.with_file(".");
        }
    }

    if globals.json {
        if let Some(directory) = config.directory.clone() {
            config = config.with_json_file(directory);
        }
    }

    Ok(config)
}
