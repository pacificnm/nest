//! Maps sqlx errors to [`nest_data::DataError`].

use nest_data::{DataError, DataErrorKind, DataResult};
use sqlx::error::ErrorKind as SqlxErrorKind;
use sqlx::Error as SqlxError;

/// Converts a sqlx error into a data error.
pub fn map_sqlx_error(error: SqlxError) -> DataError {
    let kind = match &error {
        SqlxError::RowNotFound => DataErrorKind::NotFound,
        SqlxError::Database(db) if db.kind() == SqlxErrorKind::UniqueViolation => {
            DataErrorKind::Conflict
        }
        SqlxError::PoolTimedOut | SqlxError::PoolClosed | SqlxError::Configuration(_) => {
            DataErrorKind::Connection
        }
        _ => DataErrorKind::Query,
    };
    DataError::new(kind, error.to_string()).with_source(error)
}

/// Converts a sqlx result.
pub fn sqlx_result<T>(result: Result<T, SqlxError>) -> DataResult<T> {
    result.map_err(map_sqlx_error)
}
