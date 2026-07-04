//! Module registration for [`super::StatusBarService`].

use nest_core::{AppBuilder, Module, ModuleId, NestResult};

use super::config::StatusBarConfig;
use super::service::StatusBarService;

/// Module id for [`StatusBarModule`].
pub const STATUS_BAR_MODULE_ID: ModuleId = ModuleId("nest-gui-status-bar");

/// Registers [`StatusBarService`] for app-wide status messages.
pub struct StatusBarModule {
    config: StatusBarConfig,
}

impl StatusBarModule {
    /// Creates a module with default status bar settings.
    pub fn new() -> Self {
        Self {
            config: StatusBarConfig::default(),
        }
    }

    /// Sets bar height and other display options.
    pub fn config(mut self, config: StatusBarConfig) -> Self {
        self.config = config;
        self
    }
}

impl Default for StatusBarModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for StatusBarModule {
    fn id(&self) -> ModuleId {
        STATUS_BAR_MODULE_ID
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        app.register_service(StatusBarService::new(self.config.clone()))
    }
}
