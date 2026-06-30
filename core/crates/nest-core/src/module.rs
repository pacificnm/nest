//! Module configuration contract.

use crate::builder::AppBuilder;
use nest_error::NestResult;

/// Stable identifier for a Nest module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ModuleId(pub &'static str);

impl ModuleId {
    /// Returns the module id string.
    pub fn as_str(self) -> &'static str {
        self.0
    }
}

impl std::fmt::Display for ModuleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

/// A module that configures the application during startup.
///
/// Modules register services, lifecycle handlers, and extension-point types
/// through the [`AppBuilder`]. Configuration is deferred until
/// [`AppBuilder::build`] and runs in dependency order.
pub trait Module: Send + Sync + 'static {
    /// Returns the stable id for this module.
    fn id(&self) -> ModuleId;

    /// Configures the application by registering services and capabilities.
    fn configure(&self, app: &mut AppBuilder) -> NestResult<()>;

    /// Returns module ids that must be configured before this module.
    fn dependencies(&self) -> &'static [ModuleId] {
        &[]
    }
}
