//! Font Awesome icons for Nest egui applications.
//!
//! Register [`IconModule`] and use [`Icon`] constants with [`IconButton`] or
//! [`Icon::rich_text`]. Icon names follow [Font Awesome 6](https://fontawesome.com/icons).
//!
//! ```ignore
//! use nest_gui::GuiApp;
//! use nest_icon::{Icon, IconButton, IconModule};
//!
//! GuiApp::new("my-app")
//!     .module(IconModule::new())
//!     .view(MyView)
//!     .run();
//!
//! // In a view:
//! ui.add(IconButton::new(Icon::PLAY).tooltip("Play"));
//! ui.label(Icon::GEAR.rich_text(18.0));
//! ```

#![warn(missing_docs)]
#![allow(clippy::result_large_err)]

pub mod font;
pub mod icons;
mod icon;
mod module;
mod service;
mod style;
mod widget;

pub use icon::Icon;
pub use module::{IconModule, ICON_MODULE_ID};
pub use service::IconService;
pub use style::IconStyle;
pub use widget::IconButton;

pub use nest_core::{AppContext, Service};
pub use nest_error::{NestError, NestResult};
