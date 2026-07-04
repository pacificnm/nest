//! Bottom status bar for Nest GUI apps.
//!
//! Register [`StatusBarModule`], then publish messages from anywhere via
//! [`StatusBarService`]:
//!
//! ```ignore
//! ctx.service::<StatusBarService>()?.loading("Loading movies…");
//! ctx.service::<StatusBarService>()?.loaded("42 movies loaded");
//! ```
//!
//! The GUI shell renders the bar each frame via [`show_status_bar`].

mod config;
mod host;
mod kind;
mod module;
mod service;

pub use config::StatusBarConfig;
pub use host::show_status_bar;
pub use kind::StatusKind;
pub use module::{StatusBarModule, STATUS_BAR_MODULE_ID};
pub use service::{StatusBarRight, StatusBarService, StatusBarState};
