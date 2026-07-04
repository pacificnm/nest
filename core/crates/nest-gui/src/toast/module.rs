//! Module registration for [`super::ToastService`].

use nest_core::{AppBuilder, Module, ModuleId, NestResult};

use super::config::ToastConfig;
use super::service::ToastService;

/// Module id for [`ToastModule`].
pub const TOAST_MODULE_ID: ModuleId = ModuleId("nest-gui-toast");

/// Registers [`ToastService`] for app-wide toast notifications.
pub struct ToastModule {
    config: ToastConfig,
}

impl ToastModule {
    /// Creates a module with default toast settings.
    pub fn new() -> Self {
        Self {
            config: ToastConfig::default(),
        }
    }

    /// Sets toast placement and default duration.
    pub fn config(mut self, config: ToastConfig) -> Self {
        self.config = config;
        self
    }
}

impl Default for ToastModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for ToastModule {
    fn id(&self) -> ModuleId {
        TOAST_MODULE_ID
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        app.register_service(ToastService::new(self.config.clone()))
    }
}
