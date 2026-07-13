//! Shared Tauri application state.

use std::sync::{Arc, Mutex};

use nest_app::NestApp;
use nest_core::AppContext;

use crate::config::TauriRuntimeConfig;

/// Managed state for Nest + Tauri IPC commands.
pub struct NestHostState {
    /// Application display name.
    pub app_name: String,
    /// Shared Nest service registry.
    pub context: Arc<AppContext>,
    /// Merged window settings.
    pub runtime_config: TauriRuntimeConfig,
    nest_app: Mutex<NestApp>,
}

impl NestHostState {
    pub(crate) fn new(nest_app: NestApp, runtime_config: TauriRuntimeConfig) -> Self {
        let app_name = nest_app.metadata().name.clone();
        let context = nest_app.context_arc();
        Self {
            app_name,
            context,
            runtime_config,
            nest_app: Mutex::new(nest_app),
        }
    }

    /// Shuts down the Nest application container (idempotent).
    pub fn shutdown(&self) -> nest_error::NestResult<()> {
        let mut app = self
            .nest_app
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        app.shutdown()
    }
}
