//! # nest-core
//!
//! Core contracts for the [Nest](https://github.com/pacificnm/nest) modular
//! desktop application framework.
//!
//! nest-core defines the minimum application architecture: module configuration,
//! explicit singleton service registration, typed service lookup, synchronous
//! lifecycle hooks, and extension-point traits for optional crates.
//!
//! ## Design principles (v1)
//!
//! - **Small typed service registry** — not a full DI container
//! - **Explicit registration** — `register_service(instance)`, lookup via `service::<T>()?`
//! - **Singleton only** — services are `Send + Sync + 'static`
//! - **Sync lifecycle** — async execution belongs in `nest-task-runtime`
//! - **No UI frameworks, no Tokio** — core stays dependency-light
//!
//! ## Example
//!
//! ```
//! use std::sync::Arc;
//!
//! use nest_core::{AppBuilder, AppContext, Lifecycle, Module, ModuleId, NestResult};
//!
//! struct Logger;
//!
//! struct LoggingModule;
//!
//! impl Module for LoggingModule {
//!     fn id(&self) -> ModuleId {
//!         ModuleId("logging")
//!     }
//!
//!     fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
//!         app.register_service(Logger)?;
//!         Ok(())
//!     }
//! }
//!
//! struct AppLifecycle;
//!
//! impl Lifecycle for AppLifecycle {
//!     fn on_startup(&mut self, ctx: Arc<AppContext>) -> NestResult<()> {
//!         let _logger = ctx.service::<Logger>()?;
//!         Ok(())
//!     }
//! }
//!
//! let mut app = AppBuilder::new().module(LoggingModule);
//! app.register_lifecycle(AppLifecycle);
//!
//! let mut built = app.build().unwrap();
//! built.startup().unwrap();
//! built.shutdown().unwrap();
//! ```

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

mod builder;
mod context;
mod lifecycle;
mod module;
mod registry;
mod traits;
mod version;

pub use builder::{AppBuilder, BuiltApp};
pub use context::AppContext;
pub use lifecycle::Lifecycle;
pub use module::{Module, ModuleId};
pub use nest_error::{
    NestError, NestErrorFields, NestErrorKind, NestErrorReport, NestResult, NestResultExt,
};
pub use registry::ServiceRegistry;
pub use traits::{Command, Job, Panel, Plugin, RegistrationInfo, Service};
pub use version::{nest_version, NEST_VERSION};
