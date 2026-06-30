//! Result context extension for wrapping source errors.

use crate::error::NestError;
use crate::kind::NestErrorKind;

/// Result type used throughout Nest crates.
#[allow(clippy::result_large_err)]
pub type NestResult<T> = Result<T, NestError>;

/// Extension trait for adding Nest error context to `Result`.
#[allow(clippy::result_large_err)]
pub trait NestResultExt<T> {
    /// Maps an error into a [`NestError`] with the given kind and message, preserving the source.
    fn nest_context(self, kind: NestErrorKind, message: impl Into<String>) -> NestResult<T>;

    /// Maps an error using a builder function on a base [`NestError`].
    fn nest_context_with(self, builder: impl FnOnce(NestError) -> NestError) -> NestResult<T>;
}

impl<T, E> NestResultExt<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn nest_context(self, kind: NestErrorKind, message: impl Into<String>) -> NestResult<T> {
        self.map_err(|err| NestError::new(kind, message).with_source(err))
    }

    fn nest_context_with(self, builder: impl FnOnce(NestError) -> NestError) -> NestResult<T> {
        self.map_err(|err| builder(NestError::unknown("Operation failed").with_source(err)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[derive(Debug, thiserror::Error)]
    #[error("inner failure")]
    struct InnerError;

    #[test]
    fn nest_context_wraps_source() {
        let result: NestResult<()> = Err(InnerError).nest_context(NestErrorKind::Io, "read failed");

        let err = result.unwrap_err();
        assert_eq!(err.kind(), NestErrorKind::Io);
        assert_eq!(err.message(), "read failed");
        assert!(err.source().is_some());
    }

    #[test]
    fn nest_context_with_builder() {
        let result: NestResult<()> = Err(io::Error::new(io::ErrorKind::NotFound, "missing"))
            .nest_context_with(|e| e.with_module("nest-core"));

        let err = result.unwrap_err();
        assert_eq!(err.module(), Some("nest-core"));
        assert!(err.source().is_some());
    }
}
