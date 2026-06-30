//! Extension-point traits shared across Nest crates.

mod job;
mod registrable;
mod service;

pub use job::Job;
pub use registrable::RegistrationInfo;
pub use service::Service;

use crate::builder::AppBuilder;

/// A dockable or embeddable UI panel.
///
/// Implemented by `nest-ui` and `nest-docking`.
pub trait Panel: Send + Sync + 'static {
    /// Returns a stable identifier for this panel.
    fn id(&self) -> &str;
}

/// A command that can appear in palettes, menus, or toolbars.
///
/// Implemented by `nest-commands`.
pub trait Command: Send + Sync + 'static {
    /// Returns a stable identifier for this command.
    fn id(&self) -> &str;

    /// Returns the display title for this command.
    fn title(&self) -> &str;
}

/// A plugin that registers capabilities with the application.
///
/// Implemented by `nest-plugins` and feature modules such as `nest-git`.
pub trait Plugin: Send + Sync + 'static {
    /// Registers the plugin's services, panels, commands, and other capabilities.
    fn register(&self, app: &mut AppBuilder);
}
