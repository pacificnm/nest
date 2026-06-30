# API reference

Quick reference for the nest-core v1 public API.

**Crate:** `nest_core`  
**Version:** 0.1.0  
**MSRV:** 1.75

## Re-exports

```rust
pub use builder::{AppBuilder, BuiltApp};
pub use context::AppContext;
pub use error::{NestError, NestResult};
pub use lifecycle::Lifecycle;
pub use module::Module;
pub use registry::ServiceRegistry;
pub use traits::{Command, Job, Panel, Plugin, RegistrationInfo, Service};
pub use version::{nest_version, NEST_VERSION};
```

---

## AppBuilder

| Method | Signature | Notes |
|--------|-----------|-------|
| `new` | `fn new() -> Self` | Create builder |
| `default` | `fn default() -> Self` | Same as `new` |
| `module` | `fn module<M: Module>(self, module: M) -> Self` | Eager configure |
| `register_service` | `fn register_service<T: Service>(&mut self, service: T) -> NestResult<()>` | Singleton register |
| `register_lifecycle` | `fn register_lifecycle<L: Lifecycle>(&mut self, handler: L) -> &mut Self` | Add lifecycle handler |
| `register_panel` | `fn register_panel<P: Panel>(&mut self, panel: P) -> &mut Self` | v1: metadata only |
| `register_command` | `fn register_command<C: Command>(&mut self, command: C) -> &mut Self` | v1: metadata only |
| `register_job` | `fn register_job<J: Job>(&mut self, job: J) -> &mut Self` | v1: metadata only |
| `register_plugin` | `fn register_plugin<P: Plugin>(&mut self, plugin: P) -> &mut Self` | Calls `Plugin::register` |
| `panels` | `fn panels(&self) -> &[RegistrationInfo]` | Introspection |
| `commands` | `fn commands(&self) -> &[RegistrationInfo]` | Introspection |
| `jobs` | `fn jobs(&self) -> &[RegistrationInfo]` | Introspection |
| `build` | `fn build(self) -> NestResult<BuiltApp>` | Freeze registry |

---

## BuiltApp

| Field / Method | Type / Signature | Notes |
|----------------|------------------|-------|
| `context` | `Arc<AppContext>` | Shared runtime context |
| `startup` | `fn startup(&mut self) -> NestResult<()>` | Run `on_startup` hooks |
| `shutdown` | `fn shutdown(&mut self) -> NestResult<()>` | Run `on_shutdown` hooks |

---

## AppContext

| Method | Signature | Notes |
|--------|-----------|-------|
| `service` | `fn service<T: Service>(&self) -> NestResult<&T>` | Typed lookup |
| `has_service` | `fn has_service<T: Service>(&self) -> bool` | Check without error |

---

## ServiceRegistry

| Method | Signature | Notes |
|--------|-----------|-------|
| `new` | `fn new() -> Self` | Empty registry |
| `default` | `fn default() -> Self` | Same as `new` |
| `register` | `fn register<T: Service>(&mut self, service: T) -> NestResult<()>` | Add singleton |
| `get` | `fn get<T: Service>(&self) -> NestResult<&T>` | Typed lookup |
| `contains` | `fn contains<T: Service>(&self) -> bool` | Check registration |

---

## Traits

### Module

```rust
pub trait Module: Send + Sync + 'static {
    fn configure(&self, app: &mut AppBuilder);
}
```

### Lifecycle

```rust
pub trait Lifecycle: Send + 'static {
    fn on_startup(&mut self, ctx: Arc<AppContext>) -> NestResult<()>;
    fn on_shutdown(&mut self, ctx: Arc<AppContext>) -> NestResult<()>;
}
```

Default implementations return `Ok(())`.

### Service

```rust
pub trait Service: Send + Sync + 'static {}
// Auto-implemented for all qualifying types
```

### Panel

```rust
pub trait Panel: Send + Sync + 'static {
    fn id(&self) -> &str;
}
```

### Command

```rust
pub trait Command: Send + Sync + 'static {
    fn id(&self) -> &str;
    fn title(&self) -> &str;
}
```

### Job

```rust
pub trait Job: Send + Sync + 'static {
    fn id(&self) -> &str;
}
```

### Plugin

```rust
pub trait Plugin: Send + Sync + 'static {
    fn register(&self, app: &mut AppBuilder);
}
```

---

## RegistrationInfo

```rust
pub struct RegistrationInfo {
    pub id: String,
    pub type_id: TypeId,
    pub type_name: String,
}

impl RegistrationInfo {
    pub fn new<T: 'static>(id: impl Into<String>) -> Self;
}
```

---

## Errors

```rust
pub type NestResult<T> = Result<T, NestError>;

pub enum NestError {
    ServiceNotFound(&'static str),
    ServiceAlreadyRegistered(&'static str),
    ModuleError(String),
    LifecycleError(String),
    Other(String),
}
```

---

## Version

```rust
pub const NEST_VERSION: &str;
pub fn nest_version() -> &'static str;
```

---

## Type bounds summary

| Type | Bounds |
|------|--------|
| Service | `Send + Sync + 'static` |
| Module | `Send + Sync + 'static` |
| Lifecycle handler | `Send + 'static` |
| Panel, Command, Job, Plugin | `Send + Sync + 'static` |
