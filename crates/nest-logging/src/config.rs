//! Logging configuration and builder.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::format::LogFormat;
use crate::level::LogLevel;
use crate::retention::RetentionPolicy;
use crate::rotation::RotationPolicy;
use crate::target::LogTarget;

/// Configuration for nest-logging initialization.
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Application name used in log file prefixes.
    pub app_name: String,
    /// Default log level when no module override matches.
    pub level: LogLevel,
    /// Per-target log level overrides (tracing target / module path).
    pub module_levels: HashMap<String, LogLevel>,
    /// Output destinations.
    pub targets: Vec<LogTarget>,
    /// Default format for console and text file output.
    pub format: LogFormat,
    /// Directory for file and JSON file targets.
    pub directory: Option<PathBuf>,
    /// Old log file retention policy.
    pub retention: RetentionPolicy,
    /// File rotation policy.
    pub rotation: RotationPolicy,
    /// Install a panic hook that logs via tracing.
    pub capture_panics: bool,
    /// When true, `RUST_LOG` overrides configured module levels.
    pub env_override: bool,
}

impl LoggingConfig {
    /// Creates a new logging configuration for the given application name.
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
            level: LogLevel::Info,
            module_levels: HashMap::new(),
            targets: Vec::new(),
            format: LogFormat::default(),
            directory: None,
            retention: RetentionPolicy::default(),
            rotation: RotationPolicy::default(),
            capture_panics: false,
            env_override: true,
        }
    }

    /// Enables console output.
    pub fn with_console(mut self) -> Self {
        if !self.targets.contains(&LogTarget::Console) {
            self.targets.push(LogTarget::Console);
        }
        self
    }

    /// Enables rolling text file output in `directory`.
    pub fn with_file(mut self, directory: impl Into<PathBuf>) -> Self {
        self.directory = Some(directory.into());
        if !self.targets.contains(&LogTarget::File) {
            self.targets.push(LogTarget::File);
        }
        self
    }

    /// Enables rolling JSON file output in `directory`.
    pub fn with_json_file(mut self, directory: impl Into<PathBuf>) -> Self {
        self.directory = Some(directory.into());
        if !self.targets.contains(&LogTarget::JsonFile) {
            self.targets.push(LogTarget::JsonFile);
        }
        self
    }

    /// Sets the default log level.
    pub fn with_default_level(mut self, level: LogLevel) -> Self {
        self.level = level;
        self
    }

    /// Sets a per-target log level override.
    pub fn with_module_level(mut self, target: impl Into<String>, level: LogLevel) -> Self {
        self.module_levels.insert(target.into(), level);
        self
    }

    /// Sets the output format for console and text file layers.
    pub fn with_format(mut self, format: LogFormat) -> Self {
        self.format = format;
        self
    }

    /// Sets the file rotation policy.
    pub fn with_rotation(mut self, policy: RotationPolicy) -> Self {
        self.rotation = policy;
        self
    }

    /// Sets the log file retention policy.
    pub fn with_retention(mut self, policy: RetentionPolicy) -> Self {
        self.retention = policy;
        self
    }

    /// Enables or disables panic capture via tracing.
    pub fn capture_panics(mut self, enabled: bool) -> Self {
        self.capture_panics = enabled;
        self
    }

    /// Enables or disables `RUST_LOG` environment override.
    pub fn env_override(mut self, enabled: bool) -> Self {
        self.env_override = enabled;
        self
    }

    /// Returns whether console output is enabled.
    pub fn has_console(&self) -> bool {
        self.targets.contains(&LogTarget::Console)
    }

    /// Returns whether text file output is enabled.
    pub fn has_file(&self) -> bool {
        self.targets.contains(&LogTarget::File)
    }

    /// Returns whether JSON file output is enabled.
    pub fn has_json_file(&self) -> bool {
        self.targets.contains(&LogTarget::JsonFile)
    }
}
