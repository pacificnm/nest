//! # nest-error
//!
//! Shared error model for the [Nest](https://github.com/pacificnm/nest) framework.
//!
//! nest-error defines the error shape used across all Nest crates: structured
//! [`NestError`] values with [`NestErrorKind`], stable codes, module/operation
//! metadata, source chaining, and UI-ready [`NestErrorReport`] snapshots.
//!
//! ## Design principles (v1)
//!
//! - **Struct + kind** — consistent metadata for UI, CLI, logs, and plugins
//! - **Explicit codes** — stable `NEST_*` constants for diagnostics
//! - **Source chaining** — wrap lower-level errors via [`NestResultExt`]
//! - **Logging hooks** — [`NestError::fields`] for future `nest-logging` (no tracing dep here)
//!
//! ## Example
//!
//! ```
//! use nest_error::prelude::*;
//!
//! fn load_config(path: &str) -> NestResult<String> {
//!     std::fs::read_to_string(path).nest_context(
//!         NestErrorKind::Config,
//!         "Failed to read application config",
//!     )
//! }
//!
//! fn validate_email(email: &str) -> NestResult<()> {
//!     if email.is_empty() {
//!         return Err(
//!             NestError::validation("Email is required")
//!                 .with_code("NEST_VALIDATION_REQUIRED")
//!                 .with_module("nest-forms")
//!                 .with_help("Enter an email before saving."),
//!         );
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Logging integration (future nest-logging)
//!
//! nest-error owns error shape; nest-logging will emit structured events using
//! accessors such as `error.kind()`, `error.code()`, and `error.fields()`:
//!
//! ```ignore
//! tracing::error!(
//!     kind = ?error.kind(),
//!     code = error.code(),
//!     module = error.module(),
//!     operation = error.operation(),
//!     help = error.help(),
//!     error = %error,
//!     "Nest error occurred"
//! );
//! ```

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

pub mod codes;
mod context;
mod error;
mod kind;
pub mod prelude;
mod report;

pub use context::{NestResult, NestResultExt};
pub use error::{NestError, NestErrorFields};
pub use kind::NestErrorKind;
pub use report::NestErrorReport;
