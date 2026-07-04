//! Log level mapping between Nest and tracing.

use std::fmt;
use std::str::FromStr;

/// Nest log level, mapped to [`tracing::Level`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogLevel {
    /// Trace-level diagnostics.
    Trace,
    /// Debug-level diagnostics.
    Debug,
    /// Informational messages.
    Info,
    /// Warning messages.
    Warn,
    /// Error messages.
    Error,
}

impl LogLevel {
    /// Returns the tracing level equivalent.
    pub fn to_tracing(self) -> tracing::Level {
        match self {
            Self::Trace => tracing::Level::TRACE,
            Self::Debug => tracing::Level::DEBUG,
            Self::Info => tracing::Level::INFO,
            Self::Warn => tracing::Level::WARN,
            Self::Error => tracing::Level::ERROR,
        }
    }

    /// Maps a tracing level to the Nest log level.
    pub fn from_tracing(level: tracing::Level) -> Self {
        match level {
            tracing::Level::TRACE => Self::Trace,
            tracing::Level::DEBUG => Self::Debug,
            tracing::Level::INFO => Self::Info,
            tracing::Level::WARN => Self::Warn,
            tracing::Level::ERROR => Self::Error,
        }
    }

    /// Returns the EnvFilter directive prefix for this level.
    pub fn directive(self) -> &'static str {
        match self {
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}

impl From<LogLevel> for tracing::Level {
    fn from(level: LogLevel) -> Self {
        level.to_tracing()
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.directive())
    }
}

impl FromStr for LogLevel {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "trace" => Ok(Self::Trace),
            "debug" => Ok(Self::Debug),
            "info" => Ok(Self::Info),
            "warn" | "warning" => Ok(Self::Warn),
            "error" => Ok(Self::Error),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_to_tracing_level() {
        assert_eq!(LogLevel::Info.to_tracing(), tracing::Level::INFO);
    }

    #[test]
    fn parses_level_strings() {
        assert_eq!("debug".parse(), Ok(LogLevel::Debug));
        assert_eq!("WARN".parse(), Ok(LogLevel::Warn));
    }
}
