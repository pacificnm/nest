# Modules & plugins

nest-core organizes application features through **modules** and **plugins**. Both register capabilities on `AppBuilder` during the configuration phase.

## Module

A module is a unit of configuration. It implements the `Module` trait:

```rust
pub trait Module: Send + Sync + 'static {
    fn id(&self) -> ModuleId;
    fn configure(&self, app: &mut AppBuilder) -> NestResult<()>;
    fn dependencies(&self) -> &'static [ModuleId] {
        &[]
    }
}
```

Every optional Nest capability (themes, validation, logging, etc.) integrates through this same **Module + Service Registry** pattern.

### Responsibilities

A module typically:

- Registers singleton services
- Registers lifecycle handlers
- Registers panels, commands, and jobs (v1: metadata only)
- Delegates to sub-modules or plugins

### Example

```rust
struct DataModule;

impl Module for DataModule {
    fn id(&self) -> ModuleId {
        ModuleId("my-app-data")
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        app.register_service(SqliteProvider::new())?;
        app.register_service(MigrationRunner::new())?;
        app.register_lifecycle(DatabaseLifecycle);
        Ok(())
    }
}
```

### Registration timing

`AppBuilder::module` **records** the module. Configuration runs when [`AppBuilder::build`](application.md) is called, in dependency order:

```rust
let built = AppBuilder::new()
    .module(LoggingModule)   // recorded
    .module(DataModule)      // recorded
    .build()?;               // configure runs here (deps first)
```

### Module ordering and dependencies

Declare dependencies with `Module::dependencies()`. nest-core verifies all required module ids are registered and configures modules in topological order — registration order does not matter:

```rust
impl Module for CustomerModule {
    fn id(&self) -> ModuleId {
        ModuleId("my-app-customer")
    }

    fn dependencies(&self) -> &'static [ModuleId] {
        &[nest_validation::VALIDATION_MODULE_ID]
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        app.service_mut::<ValidatorRegistry>()?
            .register(CustomerNumberValidator)?;
        Ok(())
    }
}
```

Missing dependencies fail at build with `NEST_MODULE_DEPENDENCY_MISSING`.

### Extending services during configure

Use `AppBuilder::service_mut::<T>()` to mutate a service registered by another module during the configure phase. After `build()`, services are immutable via `AppContext::service::<T>()`.

## Plugin

A plugin is a higher-level registration unit for feature bundles. It implements the `Plugin` trait:

```rust
pub trait Plugin: Send + Sync + 'static {
    fn register(&self, app: &mut AppBuilder);
}
```

### Plugin vs module

| | Module | Plugin |
|---|--------|--------|
| Trait | `Module::configure` | `Plugin::register` |
| Typical use | Core app structure | Optional feature bundles |
| Registration | `app.module(X)` | `app.register_plugin(X)` |
| In README | `App::new().module(UiModule)` | `GitPlugin` example |

In practice, both do the same thing — register on `AppBuilder`. The distinction is semantic: modules are structural; plugins are optional add-ons.

### Example

```rust
pub struct GitPlugin;

impl Plugin for GitPlugin {
    fn register(&self, app: &mut AppBuilder) {
        app.register_service(GitService::new()).unwrap();
        app.register_panel(GitPanel { id: "git".into() });
        app.register_command(OpenRepository {
            id: "git.open".into(),
            title: "Open Repository".into(),
        });
    }
}
```

```rust
let mut app = AppBuilder::new();
app.register_plugin(GitPlugin);
```

### Module wrapping a plugin

A module can register plugins internally:

```rust
struct GitModule;

impl Module for GitModule {
    fn configure(&self, app: &mut AppBuilder) {
        app.register_plugin(GitPlugin);
        app.register_plugin(GitBlamePlugin);
    }
}
```

## Organizing a real application

A typical Nest application chains core modules, then optional feature modules:

```rust
let app = AppBuilder::new()
    .module(LoggingModule)
    .module(SettingsModule)
    .module(UiModule)
    .module(ThemeModule)
    .module(DockingModule)
    .module(DataModule)
    .module(GitModule)
    .module(TasksModule);
```

Each module owns its own services and registration. Applications only compile the modules they need, matching Nest's "only load what you use" principle.

## Error handling in modules

`configure` returns `NestResult<()>`. Propagate registration failures with `?`:

```rust
impl Module for MyModule {
    fn id(&self) -> ModuleId {
        ModuleId("my-app-my-module")
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        app.register_service(MyService::new())?;
        Ok(())
    }
}
```

Build fails if any module `configure` returns an error or if a declared dependency is missing.

## Dynamic plugin loading

nest-core v1 does **not** load `.so` / DLL plugins at runtime. `Plugin` is a compile-time registration trait. Dynamic loading is planned for `nest-plugins`.

## Testing modules

Test module configuration by building an app and checking services:

```rust
#[test]
fn data_module_registers_provider() {
    let built = AppBuilder::new()
        .module(DataModule)
        .build()
        .unwrap();

    assert!(built.context.has_service::<SqliteProvider>());
}
```
