//! Application build orchestration.

use nest_core::BuiltApp;
use nest_error::{NestError, NestResult};

use crate::app::NestApp;
use crate::metadata::AppMetadata;

/// Validates metadata and materializes a [`NestApp`] from a built core application.
pub struct AppBootstrapper;

impl AppBootstrapper {
    /// Builds a [`NestApp`] from metadata and a configured [`nest_core::BuiltApp`].
    pub fn build(metadata: AppMetadata, built: BuiltApp) -> NestResult<NestApp> {
        Self::validate_metadata(&metadata)?;
        Ok(NestApp::new(metadata, built))
    }

    fn validate_metadata(metadata: &AppMetadata) -> NestResult<()> {
        if metadata.name.trim().is_empty() {
            return Err(NestError::validation("application name must not be empty"));
        }
        Ok(())
    }
}
