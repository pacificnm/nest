//! Merge logging defaults, config section, and GUI startup flags.

use nest_config::ConfigDocument;
use nest_error::NestResult;
use nest_logging::{LogLevel, LoggingConfig};
use serde::Deserialize;

use crate::startup::GuiStartupOptions;

#[derive(Debug, Deserialize)]
struct LoggingSection {
    level: Option<String>,
    directory: Option<String>,
}

/// Builds the final logging configuration for GUI startup.
///
/// GUI hosts default to file logging only while the window is active.
pub fn build_logging_config(
    base: LoggingConfig,
    document: &ConfigDocument,
    startup: &GuiStartupOptions,
) -> NestResult<LoggingConfig> {
    let mut config = base.without_console();

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

    if startup.debug {
        config.level = LogLevel::Debug;
    }

    if let Some(level) = startup.log_level {
        config.level = level;
    }

    if let Some(log_file) = &startup.log_file {
        let parent = log_file.parent().filter(|path| !path.as_os_str().is_empty());
        if let Some(directory) = parent {
            config = config.with_file(directory);
        } else {
            config = config.with_file(".");
        }
    }

    config = config.without_console();
    Ok(config)
}
