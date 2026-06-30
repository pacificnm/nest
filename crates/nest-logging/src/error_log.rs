//! Structured error logging via tracing.

use nest_error::{NestError, NestResult};

/// Logs a [`NestError`] as a structured tracing error event.
pub fn log_error(error: &NestError) {
    tracing::error!(
        target: "nest_error",
        kind = ?error.kind(),
        code = error.code(),
        module = error.module(),
        operation = error.operation(),
        help = error.help(),
        error = %error,
        "Nest error occurred"
    );
}

/// Logs the error if `result` is `Err`.
pub fn log_result<T>(result: &NestResult<T>) {
    if let Err(error) = result {
        log_error(error);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nest_error::NestError;
    use tracing_subscriber::util::SubscriberInitExt;

    #[test]
    fn log_error_emits_tracing_event() {
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .without_time()
            .with_test_writer()
            .finish();

        let _ = subscriber.set_default();

        let error = NestError::validation("bad input")
            .with_code("NEST_VALIDATION")
            .with_module("nest-forms");

        log_error(&error);
    }
}
