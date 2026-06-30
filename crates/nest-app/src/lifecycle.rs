//! Application startup and shutdown orchestration.

use nest_error::{NestError, NestResult};
use tracing::{debug, info};

use crate::app::NestApp;
use crate::codes::{NEST_APP_ALREADY_STARTED, NEST_APP_NOT_STARTED};

/// Runs synchronous lifecycle hooks on a [`NestApp`].
pub struct AppLifecycleRunner;

impl AppLifecycleRunner {
    /// Runs startup lifecycle hooks when the app is not already started.
    pub fn startup(app: &mut NestApp) -> NestResult<()> {
        if app.is_started() {
            return Err(
                NestError::validation("application is already started")
                    .with_code(NEST_APP_ALREADY_STARTED),
            );
        }

        let name = app.metadata().name.clone();
        let version = app.metadata().version.clone();
        let environment = app.metadata().environment.label();
        info!(
            app.name = %name,
            app.version = version.as_deref().unwrap_or(""),
            app.environment = environment,
            "app startup"
        );

        app.built_mut().startup()?;
        app.set_started(true);
        debug!(app.name = %name, "app startup complete");
        Ok(())
    }

    /// Runs shutdown lifecycle hooks. No-op when the app was never started.
    pub fn shutdown(app: &mut NestApp) -> NestResult<()> {
        if !app.is_started() {
            return Ok(());
        }

        let name = app.metadata().name.clone();
        info!(app.name = %name, "app shutdown");

        app.built_mut().shutdown()?;
        app.set_started(false);
        debug!(app.name = %name, "app shutdown complete");
        Ok(())
    }

    /// Ensures the application has been started.
    pub fn ensure_started(app: &NestApp) -> NestResult<()> {
        if !app.is_started() {
            return Err(
                NestError::validation("application has not been started")
                    .with_code(NEST_APP_NOT_STARTED),
            );
        }
        Ok(())
    }
}
