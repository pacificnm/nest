//! Application builder and built application state.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use crate::context::AppContext;
use crate::lifecycle::Lifecycle;
use crate::module::{Module, ModuleId};
use crate::registry::ServiceRegistry;
use crate::traits::{Command, Job, Panel, Plugin, RegistrationInfo, Service};
use nest_error::{NestError, NestResult};

/// Fluent builder for configuring a Nest application.
///
/// Used during module configuration to register services and extension points.
/// Call [`AppBuilder::build`] to produce a [`BuiltApp`] with a frozen
/// [`AppContext`].
pub struct AppBuilder {
    services: ServiceRegistry,
    pending_modules: Vec<Box<dyn Module>>,
    lifecycle_handlers: Vec<Box<dyn Lifecycle>>,
    panels: Vec<RegistrationInfo>,
    commands: Vec<RegistrationInfo>,
    jobs: Vec<RegistrationInfo>,
}

impl AppBuilder {
    /// Creates a new application builder.
    pub fn new() -> Self {
        Self {
            services: ServiceRegistry::new(),
            pending_modules: Vec::new(),
            lifecycle_handlers: Vec::new(),
            panels: Vec::new(),
            commands: Vec::new(),
            jobs: Vec::new(),
        }
    }

    /// Registers a module for configuration at build time.
    ///
    /// Modules are configured in dependency order when [`AppBuilder::build`] is
    /// called. Duplicate module ids are rejected at build time.
    pub fn module<M: Module + 'static>(mut self, module: M) -> Self {
        self.pending_modules.push(Box::new(module));
        self
    }

    /// Registers a singleton service instance.
    pub fn register_service<T: Service>(&mut self, service: T) -> NestResult<()> {
        self.services.register(service)
    }

    /// Returns a mutable reference to a registered service during configuration.
    pub fn service_mut<T: Service>(&mut self) -> NestResult<&mut T> {
        self.services.get_mut::<T>()
    }

    /// Registers a lifecycle handler.
    pub fn register_lifecycle<L: Lifecycle + 'static>(&mut self, handler: L) -> &mut Self {
        self.lifecycle_handlers.push(Box::new(handler));
        self
    }

    /// Registers a UI panel for introspection (v1: collect-only).
    pub fn register_panel<P: Panel>(&mut self, panel: P) -> &mut Self {
        self.panels
            .push(RegistrationInfo::new::<P>(panel.id().to_string()));
        self
    }

    /// Registers a command for introspection (v1: collect-only).
    pub fn register_command<C: Command>(&mut self, command: C) -> &mut Self {
        self.commands
            .push(RegistrationInfo::new::<C>(command.id().to_string()));
        self
    }

    /// Registers a background job type for introspection (v1: collect-only).
    pub fn register_job<J: Job>(&mut self, job: J) -> &mut Self {
        self.jobs
            .push(RegistrationInfo::new::<J>(job.id().to_string()));
        self
    }

    /// Registers a plugin, delegating to [`Plugin::register`].
    pub fn register_plugin<P: Plugin>(&mut self, plugin: P) -> &mut Self {
        plugin.register(self);
        self
    }

    /// Returns registered panel metadata (v1 introspection).
    pub fn panels(&self) -> &[RegistrationInfo] {
        &self.panels
    }

    /// Returns registered command metadata (v1 introspection).
    pub fn commands(&self) -> &[RegistrationInfo] {
        &self.commands
    }

    /// Returns registered job metadata (v1 introspection).
    pub fn jobs(&self) -> &[RegistrationInfo] {
        &self.jobs
    }

    /// Builds the application, freezing the service registry into an [`AppContext`].
    pub fn build(mut self) -> NestResult<BuiltApp> {
        self.configure_modules()?;
        let context = Arc::new(AppContext::new(self.services));
        Ok(BuiltApp {
            context,
            lifecycle_handlers: self.lifecycle_handlers,
        })
    }

    fn configure_modules(&mut self) -> NestResult<()> {
        let modules = std::mem::take(&mut self.pending_modules);
        if modules.is_empty() {
            return Ok(());
        }

        let mut seen = HashSet::new();
        for module in &modules {
            if !seen.insert(module.id()) {
                return Err(
                    NestError::module_error(format!("duplicate module registered: {}", module.id()))
                        .with_help("Each module id may only be registered once."),
                );
            }
        }

        let registered: HashSet<ModuleId> = modules.iter().map(|m| m.id()).collect();
        for module in &modules {
            for dep in module.dependencies() {
                if !registered.contains(dep) {
                    return Err(NestError::module_dependency_missing(
                        module.id().as_str(),
                        dep.as_str(),
                    ));
                }
            }
        }

        let order = topological_sort(&modules)?;
        for index in order {
            modules[index].configure(self)?;
        }

        Ok(())
    }
}

impl Default for AppBuilder {
    fn default() -> Self {
        Self::new()
    }
}

fn topological_sort(modules: &[Box<dyn Module>]) -> NestResult<Vec<usize>> {
    let n = modules.len();
    let id_to_index: HashMap<ModuleId, usize> = modules
        .iter()
        .enumerate()
        .map(|(index, module)| (module.id(), index))
        .collect();

    let mut in_degree = vec![0usize; n];
    let mut dependents: Vec<Vec<usize>> = vec![Vec::new(); n];

    for (index, module) in modules.iter().enumerate() {
        for dep in module.dependencies() {
            let dep_index = id_to_index.get(dep).copied().ok_or_else(|| {
                NestError::module_error(format!(
                    "internal dependency resolution failed for `{}`",
                    dep.as_str()
                ))
            })?;
            dependents[dep_index].push(index);
            in_degree[index] += 1;
        }
    }

    let mut queue: VecDeque<usize> = in_degree
        .iter()
        .enumerate()
        .filter_map(|(index, degree)| (*degree == 0).then_some(index))
        .collect();

    let mut order = Vec::with_capacity(n);
    while let Some(index) = queue.pop_front() {
        order.push(index);
        for dependent in &dependents[index] {
            in_degree[*dependent] -= 1;
            if in_degree[*dependent] == 0 {
                queue.push_back(*dependent);
            }
        }
    }

    if order.len() != n {
        return Err(
            NestError::module_error("circular module dependency detected")
                .with_help("Check Module::dependencies for cycles."),
        );
    }

    Ok(order)
}

/// A configured Nest application ready for startup.
pub struct BuiltApp {
    /// Shared application context for service lookup.
    pub context: Arc<AppContext>,
    lifecycle_handlers: Vec<Box<dyn Lifecycle>>,
}

impl BuiltApp {
    /// Runs all registered startup lifecycle hooks in registration order.
    pub fn startup(&mut self) -> NestResult<()> {
        for handler in &mut self.lifecycle_handlers {
            handler.on_startup(Arc::clone(&self.context))?;
        }
        Ok(())
    }

    /// Runs all registered shutdown lifecycle hooks in registration order.
    pub fn shutdown(&mut self) -> NestResult<()> {
        for handler in &mut self.lifecycle_handlers {
            handler.on_shutdown(Arc::clone(&self.context))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lifecycle::Lifecycle;
    use crate::module::Module;
    use crate::traits::{Command, Panel, Plugin};
    use nest_error::{codes, NestErrorKind};
    use std::sync::atomic::{AtomicUsize, Ordering};

    const LOGGING_MODULE_ID: ModuleId = ModuleId("test-logging");
    const GIT_MODULE_ID: ModuleId = ModuleId("test-git");
    const DEPENDENT_MODULE_ID: ModuleId = ModuleId("test-dependent");
    const VALIDATION_MODULE_ID: ModuleId = ModuleId("test-validation");

    struct Logger {
        name: String,
    }

    struct GitService {
        repo: String,
    }

    struct LoggingModule;

    impl Module for LoggingModule {
        fn id(&self) -> ModuleId {
            LOGGING_MODULE_ID
        }

        fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
            app.register_service(Logger {
                name: "default".to_string(),
            })
        }
    }

    struct GitModule;

    impl Module for GitModule {
        fn id(&self) -> ModuleId {
            GIT_MODULE_ID
        }

        fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
            app.register_service(GitService {
                repo: "/tmp/repo".to_string(),
            })
        }
    }

    struct TestPanel {
        panel_id: String,
    }

    impl Panel for TestPanel {
        fn id(&self) -> &str {
            &self.panel_id
        }
    }

    struct TestCommand {
        command_id: String,
        command_title: String,
    }

    impl Command for TestCommand {
        fn id(&self) -> &str {
            &self.command_id
        }

        fn title(&self) -> &str {
            &self.command_title
        }
    }

    struct TestPlugin;

    impl Plugin for TestPlugin {
        fn register(&self, app: &mut AppBuilder) {
            app.register_service(Logger {
                name: "plugin".to_string(),
            })
            .expect("plugin logger");
        }
    }

    static STARTUP_COUNT: AtomicUsize = AtomicUsize::new(0);
    static SHUTDOWN_COUNT: AtomicUsize = AtomicUsize::new(0);

    struct TestLifecycle;

    impl Lifecycle for TestLifecycle {
        fn on_startup(&mut self, ctx: Arc<AppContext>) -> NestResult<()> {
            let _logger = ctx.service::<Logger>()?;
            STARTUP_COUNT.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        fn on_shutdown(&mut self, _ctx: Arc<AppContext>) -> NestResult<()> {
            SHUTDOWN_COUNT.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[test]
    fn module_registers_service() {
        let mut app = AppBuilder::new().module(LoggingModule);
        app.register_service(GitService {
            repo: "manual".to_string(),
        })
        .unwrap();

        let built = app.build().unwrap();
        let logger = built.context.service::<Logger>().unwrap();
        assert_eq!(logger.name, "default");

        let git = built.context.service::<GitService>().unwrap();
        assert_eq!(git.repo, "manual");
    }

    #[test]
    fn lifecycle_startup_shutdown_called() {
        STARTUP_COUNT.store(0, Ordering::SeqCst);
        SHUTDOWN_COUNT.store(0, Ordering::SeqCst);

        let mut app = AppBuilder::new().module(LoggingModule).module(GitModule);
        app.register_lifecycle(TestLifecycle);

        let mut built = app.build().unwrap();
        built.startup().unwrap();
        built.shutdown().unwrap();

        assert_eq!(STARTUP_COUNT.load(Ordering::SeqCst), 1);
        assert_eq!(SHUTDOWN_COUNT.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn build_freezes_registry() {
        let built = AppBuilder::new().module(LoggingModule).build().unwrap();

        assert!(built.context.has_service::<Logger>());
        assert!(!built.context.has_service::<GitService>());
    }

    #[test]
    fn register_panel_and_command_collect_metadata() {
        let mut app = AppBuilder::new();
        app.register_panel(TestPanel {
            panel_id: "explorer".to_string(),
        });
        app.register_command(TestCommand {
            command_id: "open".to_string(),
            command_title: "Open".to_string(),
        });

        assert_eq!(app.panels().len(), 1);
        assert_eq!(app.panels()[0].id, "explorer");
        assert_eq!(app.commands().len(), 1);
        assert_eq!(app.commands()[0].id, "open");
    }

    #[test]
    fn plugin_registers_via_builder() {
        let mut builder = AppBuilder::new();
        builder.register_plugin(TestPlugin);
        let built = builder.build().unwrap();
        let logger = built.context.service::<Logger>().unwrap();
        assert_eq!(logger.name, "plugin");
    }

    #[test]
    fn duplicate_service_via_modules_fails_at_register() {
        struct DuplicateModule;
        impl Module for DuplicateModule {
            fn id(&self) -> ModuleId {
                ModuleId("test-duplicate")
            }

            fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
                app.register_service(Logger {
                    name: "first".to_string(),
                })?;
                let err = app
                    .register_service(Logger {
                        name: "second".to_string(),
                    })
                    .unwrap_err();
                assert_eq!(err.kind(), NestErrorKind::Service);
                assert_eq!(err.code(), Some(codes::NEST_SERVICE_ALREADY_REGISTERED));
                Ok(())
            }
        }

        AppBuilder::new().module(DuplicateModule).build().unwrap();
    }

    struct MarkerService {
        value: usize,
    }

    struct ValidationLikeModule;

    impl Module for ValidationLikeModule {
        fn id(&self) -> ModuleId {
            VALIDATION_MODULE_ID
        }

        fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
            app.register_service(MarkerService { value: 1 })
        }
    }

    struct DependentModule;

    impl Module for DependentModule {
        fn id(&self) -> ModuleId {
            DEPENDENT_MODULE_ID
        }

        fn dependencies(&self) -> &'static [ModuleId] {
            &[VALIDATION_MODULE_ID]
        }

        fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
            let marker = app.service_mut::<MarkerService>()?;
            marker.value += 1;
            Ok(())
        }
    }

    #[test]
    fn missing_module_dependency_fails_at_build() {
        let result = AppBuilder::new().module(DependentModule).build();
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(err.kind(), NestErrorKind::Module);
        assert_eq!(err.code(), Some(codes::NEST_MODULE_DEPENDENCY_MISSING));
    }

    #[test]
    fn modules_configure_in_dependency_order_even_if_registered_reverse() {
        let built = AppBuilder::new()
            .module(DependentModule)
            .module(ValidationLikeModule)
            .build()
            .unwrap();
        let marker = built.context.service::<MarkerService>().unwrap();
        assert_eq!(marker.value, 2);
    }

    #[test]
    fn service_mut_available_during_configure() {
        struct ExtendingModule;

        impl Module for ExtendingModule {
            fn id(&self) -> ModuleId {
                ModuleId("test-extending")
            }

            fn dependencies(&self) -> &'static [ModuleId] {
                &[VALIDATION_MODULE_ID]
            }

            fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
                let marker = app.service_mut::<MarkerService>()?;
                marker.value = 42;
                Ok(())
            }
        }

        let built = AppBuilder::new()
            .module(ExtendingModule)
            .module(ValidationLikeModule)
            .build()
            .unwrap();
        let marker = built.context.service::<MarkerService>().unwrap();
        assert_eq!(marker.value, 42);
    }
}
