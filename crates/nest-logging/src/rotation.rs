//! Log file rotation policy.

use nest_error::{NestError, NestResult};
use tracing_appender::rolling::{self, Rotation};

use crate::codes::NEST_LOGGING_ROTATION_UNSUPPORTED;

/// When log files are rotated to a new file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RotationPolicy {
    /// Never rotate; single file appender.
    #[default]
    Never,
    /// Rotate daily at midnight.
    Daily,
    /// Rotate every hour.
    Hourly,
    /// Rotate when file exceeds size — not implemented in v1.
    SizeBytes(u64),
}

/// Validates rotation policy for v1 and returns the tracing-appender rotation.
pub fn validate_rotation(policy: RotationPolicy) -> NestResult<Rotation> {
    match policy {
        RotationPolicy::Never => Ok(Rotation::NEVER),
        RotationPolicy::Daily => Ok(Rotation::DAILY),
        RotationPolicy::Hourly => Ok(Rotation::HOURLY),
        RotationPolicy::SizeBytes(_) => Err(NestError::config(
            "size-based log rotation is not supported in nest-logging v1",
        )
        .with_code(NEST_LOGGING_ROTATION_UNSUPPORTED)),
    }
}

/// Creates a rolling file appender for the given directory and application name.
pub fn file_appender(
    directory: &std::path::Path,
    app_name: &str,
    rotation: Rotation,
) -> rolling::RollingFileAppender {
    match rotation {
        Rotation::NEVER => rolling::never(directory, app_name),
        Rotation::DAILY => rolling::daily(directory, app_name),
        Rotation::HOURLY => rolling::hourly(directory, app_name),
        _ => rolling::never(directory, app_name),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codes::NEST_LOGGING_ROTATION_UNSUPPORTED;

    #[test]
    fn size_rotation_unsupported_in_v1() {
        let err = validate_rotation(RotationPolicy::SizeBytes(1024)).unwrap_err();
        assert_eq!(err.code(), Some(NEST_LOGGING_ROTATION_UNSUPPORTED));
    }
}
