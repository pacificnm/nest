//! # nest-logging
//!
//! Optional tracing-based logging infrastructure for [Nest](https://github.com/pacificnm/nest)
//! host applications.
//!
//! Feature crates should depend on `tracing` and `nest-error` only. The host app (Kiwi,
//! Nest CLI, etc.) installs `nest-logging` at startup.
//!
//! ## Example
//!
//! ```
//! use nest_logging::prelude::*;
//!
//! # fn example() -> nest_error::NestResult<()> {
//! init(
//!     LoggingConfig::new("kiwi")
//!         .with_console()
//!         .with_file("./logs")
//!         .with_default_level(LogLevel::Info)
//!         .with_module_level("nest_data", LogLevel::Debug)
//!         .with_rotation(RotationPolicy::Daily)
//!         .with_retention(RetentionPolicy::Days(14))
//!         .capture_panics(true),
//! )?;
//!
//! tracing::info!(target: "nest_data", "query executed");
//! # Ok(())
//! # }
//! ```
//!
//! ## Boundaries
//!
//! - **nest-error** creates structured errors
//! - **nest-logging** records them via [`log_error`]
//! - **nest-core** has no logging dependency

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

pub mod codes;
mod config;
mod error_log;
mod filter;
mod format;
mod init;
mod level;
mod panic_hook;
pub mod prelude;
mod retention;
mod rotation;
mod target;
mod ui_buffer;

pub use config::LoggingConfig;
pub use error_log::{log_error, log_result};
pub use format::LogFormat;
pub use init::{init, init_logging, LoggingGuard};
pub use level::LogLevel;
pub use retention::{cleanup_logs, RetentionPolicy};
pub use rotation::RotationPolicy;
pub use target::LogTarget;
pub use ui_buffer::{ui_buffer, LogBuffer, LogRecord};
