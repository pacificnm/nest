//! Facade over [`nest_core::AppBuilder`].

use nest_core::{AppBuilder, Command, Job, Lifecycle, Module, NestResult, Panel, Plugin, Service};

use crate::app::NestApp;
use crate::bootstrap::AppBootstrapper;
use crate::metadata::{AppEnvironment, AppMetadata};

/// High-level builder for a [`NestApp`] container.
pub struct NestAppBuilder {
    metadata: AppMetadata,
    core: AppBuilder,
}

impl NestAppBuilder {
    /// Creates a builder for the given application name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            metadata: AppMetadata::new(name),
            core: AppBuilder::new(),
        }
    }

    /// Sets the application version string.
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.metadata = self.metadata.with_version(version);
        self
    }

    /// Sets the deployment environment.
    pub fn environment(mut self, environment: AppEnvironment) -> Self {
        self.metadata = self.metadata.with_environment(environment);
        self
    }

    /// Registers a Nest module.
    pub fn module<M: Module + 'static>(mut self, module: M) -> Self {
        self.core = self.core.module(module);
        self
    }

    /// Registers a singleton service instance.
    pub fn register_service<T: Service>(mut self, service: T) -> NestResult<Self> {
        self.core.register_service(service)?;
        Ok(self)
    }

    /// Returns a mutable reference to a registered service during configuration.
    pub fn service_mut<T: Service>(&mut self) -> NestResult<&mut T> {
        self.core.service_mut::<T>()
    }

    /// Registers a lifecycle handler.
    pub fn register_lifecycle<L: Lifecycle + 'static>(mut self, handler: L) -> Self {
        self.core.register_lifecycle(handler);
        self
    }

    /// Registers a UI panel for introspection.
    pub fn register_panel<P: Panel>(mut self, panel: P) -> Self {
        self.core.register_panel(panel);
        self
    }

    /// Registers a command for introspection.
    pub fn register_command<C: Command>(mut self, command: C) -> Self {
        self.core.register_command(command);
        self
    }

    /// Registers a background job type for introspection.
    pub fn register_job<J: Job>(mut self, job: J) -> Self {
        self.core.register_job(job);
        self
    }

    /// Registers a plugin, delegating to [`Plugin::register`].
    pub fn register_plugin<P: Plugin>(mut self, plugin: P) -> Self {
        self.core.register_plugin(plugin);
        self
    }

    /// Builds the [`NestApp`] container.
    pub fn build(self) -> NestResult<NestApp> {
        let built = self.core.build()?;
        AppBootstrapper::build(self.metadata, built)
    }
}
