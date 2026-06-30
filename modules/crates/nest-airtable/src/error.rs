//! Airtable-specific error mapping.

use nest_error::{NestError, NestResult};
use nest_http::HttpError;

use crate::codes::{
    NEST_AIRTABLE_INVALID_RESPONSE, NEST_AIRTABLE_RATE_LIMITED, NEST_AIRTABLE_REQUEST_FAILED,
};

/// Maps an HTTP-layer error into a Nest error for Airtable operations.
pub fn http_to_airtable_error(error: NestError, operation: &str) -> NestError {
    let code = error
        .code()
        .map(str::to_string)
        .unwrap_or_else(|| NEST_AIRTABLE_REQUEST_FAILED.to_string());
    if let Some(source) = error.source().and_then(|s| s.downcast_ref::<HttpError>()) {
        if source.response_status().is_some_and(|status| status.code() == 429) {
            return NestError::network("Airtable rate limit exceeded")
                .with_code(NEST_AIRTABLE_RATE_LIMITED)
                .with_module("nest-airtable")
                .with_operation(operation)
                .with_source(error);
        }
    }

    error
        .with_module("nest-airtable")
        .with_operation(operation)
        .with_code(code)
}

/// Maps a decode/validation failure into a Nest error.
pub fn invalid_response(message: impl Into<String>, operation: &str) -> NestError {
    NestError::network(message)
        .with_code(NEST_AIRTABLE_INVALID_RESPONSE)
        .with_module("nest-airtable")
        .with_operation(operation)
}

/// Returns an error when a logical table name is missing from configuration.
pub fn table_not_found(name: &str) -> NestError {
    NestError::config(format!("Airtable table not configured: {name}"))
        .with_code(crate::codes::NEST_AIRTABLE_TABLE_NOT_FOUND)
        .with_module("nest-airtable")
        .with_help(format!(
            "Add an [airtable.tables.{name}] section with table_id."
        ))
}

/// Propagates cancellation when a token is set and cancelled.
pub fn check_cancelled(token: Option<&nest_task::CancelToken>) -> NestResult<()> {
    if token.is_some_and(nest_task::CancelToken::is_cancelled) {
        return Err(
            NestError::task("Airtable operation cancelled")
                .with_code(nest_error::codes::NEST_TASK_CANCELLED)
                .with_module("nest-airtable"),
        );
    }
    Ok(())
}
