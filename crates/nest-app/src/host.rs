//! Host integration contract.

use nest_core::AppContext;

use crate::app::NestApp;
use crate::metadata::AppMetadata;

/// Contract for hosts that consume a pre-built Nest application container.
pub trait HostApp {
    /// Returns application metadata.
    fn metadata(&self) -> &AppMetadata;

    /// Returns the shared application context.
    fn context(&self) -> &AppContext;

    /// Consumes the host app wrapper and returns the underlying container.
    fn into_nest_app(self) -> NestApp;
}

impl HostApp for NestApp {
    fn metadata(&self) -> &AppMetadata {
        self.metadata()
    }

    fn context(&self) -> &AppContext {
        self.context()
    }

    fn into_nest_app(self) -> NestApp {
        self
    }
}
