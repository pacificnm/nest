//! Theme-specific error helpers built on [`nest_error::NestError`].

use nest_error::{NestError, NestResult};

use crate::codes;

/// Creates a theme-not-found error.
pub fn theme_not_found(id: &str) -> NestError {
    NestError::validation(format!("theme not found: {id}"))
        .with_code(codes::NEST_THEME_NOT_FOUND)
        .with_operation("theme.lookup")
}

/// Creates a duplicate-theme registration error.
pub fn theme_already_registered(id: &str) -> NestError {
    NestError::validation(format!("theme already registered: {id}"))
        .with_code(codes::NEST_THEME_ALREADY_REGISTERED)
        .with_operation("theme.register")
}

/// Creates a no-active-theme error.
pub fn no_active_theme() -> NestError {
    NestError::validation("no active theme is set")
        .with_code(codes::NEST_THEME_NO_ACTIVE)
        .with_operation("theme.active")
}

/// Creates an unsupported theme format error.
pub fn unsupported_format(path: &str) -> NestError {
    NestError::config(format!("unsupported theme file format: {path}"))
        .with_code(codes::NEST_THEME_FORMAT_UNSUPPORTED)
        .with_operation("theme.load")
}

/// Wraps a validation message with field context in the message.
pub fn validation(field: &str, message: impl Into<String>) -> NestError {
    NestError::validation(format!("{field}: {}", message.into())).with_operation("theme.validate")
}

/// Converts I/O errors into Nest errors.
pub fn io(err: std::io::Error) -> NestError {
    NestError::io(err.to_string()).with_source(err)
}

/// Type alias for theme operations.
pub type ThemeResult<T> = NestResult<T>;
