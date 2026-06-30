//! Maps rusqlite errors to [`nest_data::DataError`].

use nest_data::{DataError, DataErrorKind, DataResult};

/// Converts a rusqlite error into a data error.
pub fn map_rusqlite_error(error: rusqlite::Error) -> DataError {
    let kind = match error {
        rusqlite::Error::QueryReturnedNoRows => DataErrorKind::NotFound,
        rusqlite::Error::SqliteFailure(code, _)
            if code.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            DataErrorKind::Conflict
        }
        _ => DataErrorKind::Query,
    };
    DataError::new(kind, error.to_string()).with_source(error)
}

/// Converts a rusqlite result.
pub fn sqlite_result<T>(result: rusqlite::Result<T>) -> DataResult<T> {
    result.map_err(map_rusqlite_error)
}
