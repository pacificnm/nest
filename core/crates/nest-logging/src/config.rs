//! Logging configuration and builder.

use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::sync::Arc;

use crate::format::LogFormat;
use crate::level::LogLevel;
use crate::retention::RetentionPolicy;
use crate::rotation::RotationPolicy;
use crate::target::LogTarget;
use crate::ui_buffer::LogBuffer;

/// Configuration for nest-logging initialization.
#[derive(Clone)]
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
    /// Optional in-memory buffer for GUI log panels.
    pub ui_buffer: Option<Arc<LogBuffer>>,
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
            ui_buffer: None,
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

    /// Captures filtered log events into an in-memory ring buffer for UI panels.
    pub fn with_ui_buffer(mut self, buffer: Arc<LogBuffer>) -> Self {
        self.ui_buffer = Some(buffer);
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

    /// Removes console output from the target list.
    pub fn without_console(mut self) -> Self {
        self.targets.retain(|target| *target != LogTarget::Console);
        self
    }

    /// Sensible CLI defaults: console output, info level, env override on.
    pub fn for_cli(app_name: impl Into<String>) -> Self {
        Self::new(app_name).with_console()
    }

    /// Sensible TUI defaults: file logging only (never stdout during raw-mode UI).
    pub fn for_tui(app_name: impl Into<String>) -> Self {
        Self::new(app_name).with_file("./logs")
    }

    /// Sensible GUI defaults: file logging only (console deferred to future log viewer).
    pub fn for_gui(app_name: impl Into<String>) -> Self {
        Self::new(app_name).with_file("./logs")
    }

    /// Sensible Tauri desktop defaults: file logging only, with logs outside
    /// `src-tauri/` so `tauri dev` does not rebuild on every log write.
    pub fn for_tauri(app_name: impl Into<String>) -> Self {
        Self::new(app_name).with_file("../logs")
    }
}

impl fmt::Debug for LoggingConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LoggingConfig")
            .field("app_name", &self.app_name)
            .field("level", &self.level)
            .field("module_levels", &self.module_levels)
            .field("targets", &self.targets)
            .field("format", &self.format)
            .field("directory", &self.directory)
            .field("retention", &self.retention)
            .field("rotation", &self.rotation)
            .field("capture_panics", &self.capture_panics)
            .field("env_override", &self.env_override)
            .field("ui_buffer", &self.ui_buffer.as_ref().map(|_| "LogBuffer"))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn for_cli_enables_console() {
        let config = LoggingConfig::for_cli("my-app");
        assert_eq!(config.app_name, "my-app");
        assert!(config.has_console());
        assert_eq!(config.level, LogLevel::Info);
        assert!(config.env_override);
    }

    #[test]
    fn for_tui_uses_file_only() {
        let config = LoggingConfig::for_tui("finch");
        assert!(!config.has_console());
        assert!(config.has_file());
    }

    #[test]
    fn for_gui_uses_file_only() {
        let config = LoggingConfig::for_gui("kiwi");
        assert!(!config.has_console());
        assert!(config.has_file());
    }

    #[test]
    fn for_tauri_logs_outside_src_tauri() {
        let config = LoggingConfig::for_tauri("swift");
        assert!(!config.has_console());
        assert!(config.has_file());
        assert_eq!(
            config.directory.as_deref(),
            Some(std::path::Path::new("../logs"))
        );
    }

    #[test]
    fn without_console_strips_console_target() {
        let config = LoggingConfig::for_cli("my-app").without_console();
        assert!(!config.has_console());
    }
}
