//! Host-agnostic application container for the Nest framework.
//!
//! `nest-app` wraps [`nest_core::AppBuilder`] with metadata and lifecycle
//! orchestration. Host crates (`nest-cli`, `nest-tui`, `nest-gui`) execute the
//! container; they do not replace it.

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

pub mod codes;
pub mod prelude;

mod app;
mod bootstrap;
mod builder;
mod host;
mod lifecycle;
mod metadata;

pub use app::NestApp;
pub use bootstrap::AppBootstrapper;
pub use builder::NestAppBuilder;
pub use host::HostApp;
pub use lifecycle::AppLifecycleRunner;
pub use metadata::{AppEnvironment, AppMetadata};

pub use nest_core::{AppContext, BuiltApp, Module, ModuleId, NestResult};
pub use nest_error::{NestError, NestErrorReport};

impl NestApp {
    /// Creates a new application builder for the given name.
    pub fn builder(name: impl Into<String>) -> NestAppBuilder {
        NestAppBuilder::new(name)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    use nest_core::{AppBuilder, Lifecycle, Module, ModuleId, NestResult};
    use nest_error::codes;
    use nest_validation::{ValidationModule, ValidatorRegistry, VALIDATION_MODULE_ID};

    use super::*;

    struct CounterService {
        value: usize,
    }

    struct CounterModule;

    impl Module for CounterModule {
        fn id(&self) -> ModuleId {
            ModuleId("counter")
        }

        fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
            app.register_service(CounterService { value: 42 })?;
            Ok(())
        }
    }

    static STARTUP_COUNT: AtomicUsize = AtomicUsize::new(0);

    struct TestLifecycle;

    impl Lifecycle for TestLifecycle {
        fn on_startup(&mut self, _ctx: Arc<AppContext>) -> NestResult<()> {
            STARTUP_COUNT.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    struct DependentModule;

    impl Module for DependentModule {
        fn id(&self) -> ModuleId {
            ModuleId("dependent")
        }

        fn dependencies(&self) -> &'static [ModuleId] {
            &[VALIDATION_MODULE_ID]
        }

        fn configure(&self, _app: &mut AppBuilder) -> NestResult<()> {
            Ok(())
        }
    }

    #[test]
    fn builder_forwards_module_and_service() {
        let app = NestApp::builder("kiwi")
            .version("1.0.0")
            .environment(AppEnvironment::Test)
            .module(CounterModule)
            .build()
            .unwrap();

        assert_eq!(app.metadata().name, "kiwi");
        assert_eq!(app.metadata().version.as_deref(), Some("1.0.0"));
        assert_eq!(app.metadata().environment, AppEnvironment::Test);

        let counter = app.context().service::<CounterService>().unwrap();
        assert_eq!(counter.value, 42);
    }

    #[test]
    fn dependency_validation_surfaces_core_error() {
        assert!(NestApp::builder("kiwi")
            .module(DependentModule)
            .build()
            .is_err());
    }

    #[test]
    fn lifecycle_startup_and_shutdown() {
        STARTUP_COUNT.store(0, Ordering::SeqCst);

        let mut app = NestApp::builder("kiwi")
            .register_lifecycle(TestLifecycle)
            .build()
            .unwrap();

        assert!(!app.is_started());
        app.startup().unwrap();
        assert!(app.is_started());
        assert_eq!(STARTUP_COUNT.load(Ordering::SeqCst), 1);

        app.shutdown().unwrap();
        assert!(!app.is_started());
    }

    #[test]
    fn double_startup_fails() {
        let mut app = NestApp::builder("kiwi").build().unwrap();
        app.startup().unwrap();
        let err = app.startup().unwrap_err();
        assert_eq!(err.code(), Some(codes::NEST_APP_ALREADY_STARTED));
    }

    #[test]
    fn shutdown_is_idempotent_when_not_started() {
        let mut app = NestApp::builder("kiwi").build().unwrap();
        app.shutdown().unwrap();
        app.shutdown().unwrap();
    }

    #[test]
    fn validation_module_integrates() {
        let app = NestApp::builder("kiwi")
            .module(ValidationModule::default())
            .build()
            .unwrap();
        let validators = app.context().service::<ValidatorRegistry>().unwrap();
        assert!(validators.contains("email"));
    }

    #[test]
    fn empty_name_fails_validation() {
        match NestApp::builder("   ").build() {
            Err(err) => assert_eq!(err.kind(), nest_error::NestErrorKind::Validation),
            Ok(_) => panic!("empty name should fail"),
        }
    }
}
