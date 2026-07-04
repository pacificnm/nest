//! Snackbar-style toast notifications for Nest GUI apps.
//!
//! Register [`ToastModule`], then fire messages from anywhere via
//! [`ToastService`]:
//!
//! ```ignore
//! ctx.service::<ToastService>()?.success("Movie saved");
//! ```
//!
//! The GUI shell renders active toasts each frame via [`show_toasts`].

mod config;
mod host;
mod kind;
mod module;
mod service;

pub use config::ToastConfig;
pub use host::show_toasts;
pub use kind::{ToastKind, ToastPosition};
pub use module::{ToastModule, TOAST_MODULE_ID};
pub use service::{ToastMessage, ToastService};
