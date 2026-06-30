//! Process exit code mapping.

use nest_error::{codes, NestError, NestErrorKind};

/// Standardized CLI exit codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum CliExitCode {
    /// Success.
    Success = 0,
    /// General error.
    General = 1,
    /// Validation failed.
    Validation = 2,
    /// Configuration error.
    Config = 3,
    /// File or I/O error.
    File = 4,
    /// Network or HTTP error.
    Network = 5,
    /// Data or database error.
    Data = 6,
    /// Command usage error.
    Usage = 10,
}

impl CliExitCode {
    /// Returns the process exit code as `i32`.
    pub fn as_i32(self) -> i32 {
        self as i32
    }

    /// Maps a [`NestError`] to a CLI exit code.
    pub fn from_error(error: &NestError) -> Self {
        if let Some(code) = error.code() {
            if code == codes::NEST_CLI_USAGE {
                return Self::Usage;
            }
            if code.starts_with("NEST_CONFIG_") {
                return Self::Config;
            }
            if code.starts_with("NEST_FILE_") || code.starts_with("NEST_CSV_WRITE") {
                return Self::File;
            }
            if code.starts_with("NEST_HTTP_") {
                return Self::Network;
            }
            if code.starts_with("NEST_DATA_") || code.starts_with("NEST_SQLITE_") {
                return Self::Data;
            }
        }

        match error.kind() {
            NestErrorKind::Validation => Self::Validation,
            NestErrorKind::Config => Self::Config,
            NestErrorKind::Io => Self::File,
            NestErrorKind::Network => Self::Network,
            NestErrorKind::Data => Self::Data,
            NestErrorKind::Command => Self::Usage,
            _ => Self::General,
        }
    }
}
