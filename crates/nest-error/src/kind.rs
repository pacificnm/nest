//! Error kind classification for Nest errors.

use std::fmt;

/// High-level category for a [`crate::NestError`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NestErrorKind {
    /// Configuration or settings errors.
    Config,
    /// I/O errors (files, streams, etc.).
    Io,
    /// Validation failures.
    Validation,
    /// Data layer errors.
    Data,
    /// Command execution errors.
    Command,
    /// Service registry or DI errors.
    Service,
    /// Module configuration errors.
    Module,
    /// Plugin errors.
    Plugin,
    /// Background task errors.
    Task,
    /// UI errors.
    Ui,
    /// Authentication/authorization errors.
    Auth,
    /// Network errors.
    Network,
    /// Unknown or uncategorized errors.
    Unknown,
}

impl NestErrorKind {
    /// Returns a short lowercase label for logging and CLI output.
    pub fn label(self) -> &'static str {
        match self {
            Self::Config => "config",
            Self::Io => "io",
            Self::Validation => "validation",
            Self::Data => "data",
            Self::Command => "command",
            Self::Service => "service",
            Self::Module => "module",
            Self::Plugin => "plugin",
            Self::Task => "task",
            Self::Ui => "ui",
            Self::Auth => "auth",
            Self::Network => "network",
            Self::Unknown => "unknown",
        }
    }

    /// Returns a human-readable title for error reports.
    pub fn title(self) -> &'static str {
        match self {
            Self::Config => "Configuration Error",
            Self::Io => "I/O Error",
            Self::Validation => "Validation Error",
            Self::Data => "Data Error",
            Self::Command => "Command Error",
            Self::Service => "Service Error",
            Self::Module => "Module Error",
            Self::Plugin => "Plugin Error",
            Self::Task => "Task Error",
            Self::Ui => "UI Error",
            Self::Auth => "Authentication Error",
            Self::Network => "Network Error",
            Self::Unknown => "Unknown Error",
        }
    }
}

impl fmt::Display for NestErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}
