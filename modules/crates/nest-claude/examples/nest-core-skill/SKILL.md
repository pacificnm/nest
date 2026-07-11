---
name: nest-core
description: How to build applications on nest-core, the foundation crate of the Nest Rust framework — Module registration, the typed service registry, AppBuilder/AppContext/BuiltApp, and synchronous Lifecycle hooks. Use whenever writing, reviewing, or debugging Rust code that defines a Nest `Module`, registers or looks up a service, wires up `AppBuilder`, or adds startup/shutdown behavior in a Nest-based app or crate.
---

# nest-core

`nest-core` is the foundation crate of the Nest framework (a modular Rust
application framework). Crate path: `core/crates/nest-core`. Every other Nest
crate (core or module) builds on the same three primitives it defines:

1. **How do features register themselves?** — `Module` + `AppBuilder`.
2. **How do components share state?** — an explicit, typed singleton
   `ServiceRegistry`, looked up through `AppContext`.
3. **When does setup/teardown run?** — synchronous `Lifecycle` hooks.

nest-core is deliberately small: no UI framework, no Tokio, no dynamic plugin
loading, no dependency-injection auto-wiring. Its only external dependency is
`thiserror` (used to build `nest-error`'s `NestError`).

## When this skill applies

- Defining a new `Module` for a Nest crate (core or `modules/crates/*`).
- Registering, looking up, or debugging a `Service` in the registry.
- Wiring `AppBuilder`/`AppContext`/`BuiltApp` in an app's `main.rs` or a
  library entry point.
- Adding `Lifecycle` startup/shutdown behavior.
- Diagnosing a `NestError` raised by module configuration (duplicate module
  id, missing dependency, service not found, service already registered).

Do **not** reach for this skill for anything Tokio/async, UI rendering, or
dynamic plugin loading — nest-core explicitly excludes those (see
"What nest-core does not do" below).

## Core types (verified against `core/crates/nest-core/src/*.rs`)

```rust
// module.rs
pub struct ModuleId(pub &'static str); // Copy, Eq, Hash

pub trait Module: Send + Sync + 'static {
    fn id(&self) -> ModuleId;
    fn configure(&self, app: &mut AppBuilder) -> NestResult<()>;
    fn dependencies(&self) -> &'static [ModuleId] { &[] } // default: no deps
}

// traits/service.rs — auto-implemented, not hand-written
pub trait Service: Send + Sync + 'static {}

// lifecycle.rs
pub trait Lifecycle: Send + 'static {
    fn on_startup(&mut self, ctx: Arc<AppContext>) -> NestResult<()>;
    fn on_shutdown(&mut self, ctx: Arc<AppContext>) -> NestResult<()>;
}

// builder.rs
impl AppBuilder {
    pub fn new() -> Self;
    pub fn module<M: Module + 'static>(mut self, module: M) -> Self; // chainable
    pub fn register_service<T: Service>(&mut self, service: T) -> NestResult<()>;
    pub fn service_mut<T: Service>(&mut self) -> NestResult<&mut T>; // configure-time only
    pub fn register_lifecycle<L: Lifecycle + 'static>(&mut self, handler: L) -> &mut Self;
    pub fn register_panel<P: Panel>(&mut self, panel: P) -> &mut Self;   // v1: metadata only
    pub fn register_command<C: Command>(&mut self, command: C) -> &mut Self; // v1: metadata only
    pub fn register_job<J: Job>(&mut self, job: J) -> &mut Self;         // v1: metadata only
    pub fn register_plugin<P: Plugin>(&mut self, plugin: P) -> &mut Self; // calls Plugin::register
    pub fn build(self) -> NestResult<BuiltApp>;
}

pub struct BuiltApp {
    pub context: Arc<AppContext>,
    // + startup()/shutdown() run registered Lifecycle handlers
}

impl AppContext {
    pub fn service<T: Service>(&self) -> NestResult<&T>;   // typed lookup, frozen after build
    pub fn has_service<T: Service>(&self) -> bool;
}
```

`Service` has a **blanket impl** — any `Send + Sync + 'static` type qualifies
automatically. You never write `impl Service for Foo {}` yourself.

## Quick start

```rust
use std::sync::Arc;
use nest_core::{AppBuilder, AppContext, Lifecycle, Module, ModuleId, NestResult};

struct Logger;

struct LoggingModule;

impl Module for LoggingModule {
    fn id(&self) -> ModuleId {
        ModuleId("my-app-logging")
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        app.register_service(Logger)?;
        Ok(())
    }
}

struct AppLifecycle;

impl Lifecycle for AppLifecycle {
    fn on_startup(&mut self, ctx: Arc<AppContext>) -> NestResult<()> {
        let _logger = ctx.service::<Logger>()?;
        Ok(())
    }
}

let mut app = AppBuilder::new().module(LoggingModule);
app.register_lifecycle(AppLifecycle);

let mut built = app.build().unwrap();
built.startup().unwrap();
// ... application runs ...
built.shutdown().unwrap();
```

## Module dependency ordering

`AppBuilder::build()` runs `Module::configure` for every registered module in
**dependency order**, not registration order — it topologically sorts by
`Module::dependencies()` before configuring anything. This means:

```rust
struct DependentModule;
impl Module for DependentModule {
    fn id(&self) -> ModuleId { ModuleId("dependent") }
    fn dependencies(&self) -> &'static [ModuleId] { &[ModuleId("validation")] }
    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        // Safe: `validation` module already ran, even if registered *after*
        // this one — `service_mut` can reach the service it created.
        let marker = app.service_mut::<MarkerService>()?;
        marker.value += 1;
        Ok(())
    }
}
```

- Registering `DependentModule` before `ValidationLikeModule` still configures
  validation first, because `build()` sorts by the declared dependency graph.
- A missing dependency (a `ModuleId` in `dependencies()` that no registered
  module declares as its `id()`) fails `build()` with `NestErrorKind::Module`,
  code `NEST_MODULE_DEPENDENCY_MISSING`.
- A duplicate module id (two modules registered with the same `id()`) fails
  `build()` the same way, before any module's `configure` runs.
- A circular dependency fails `build()` with a `"circular module dependency
  detected"` message.

This is why every module in this repo (`nest-http-client`, `nest-tmdb`,
`nest-airtable`, `nest-claude`, …) declares a `pub const <NAME>_MODULE_ID:
ModuleId` and lists the module ids of any service it needs via
`dependencies()` — see `core/crates/nest-http-client/src/module.rs` for the
smallest real example, or any module under `modules/crates/*/src/module.rs`.

## Common pitfalls

- **`service_mut::<T>()` only works during `configure()`** — it's a method on
  `AppBuilder`, not `AppContext`. After `build()` freezes the registry into
  `AppContext`, only the immutable `service::<T>()` lookup is available; you
  cannot register or mutate services post-build.
- **Registering the same service type twice fails immediately** at the
  `register_service` call site (`NestErrorKind::Service`, code
  `NEST_SERVICE_ALREADY_REGISTERED`) — it does not silently overwrite.
- **Modules aren't automatically wired to services** — there's no
  constructor injection. A module's `configure()` must explicitly call
  `app.register_service(...)` for every service it wants available, and
  explicitly declare `dependencies()` for every other module's service it
  reads via `service_mut`.
- **`Panel`/`Command`/`Job` registration is metadata-only in v1** — collected
  into `AppBuilder::panels()/commands()/jobs()` for introspection; nothing
  executes them. Don't expect a registered `Job` to run automatically.
- **Lifecycle hooks are synchronous** — `on_startup`/`on_shutdown` return
  `NestResult<()>`, not a future. Background/async work belongs in
  `nest-task-runtime` (`TaskManagerService`), invoked *from* a lifecycle hook
  if needed, not implemented as one.

## What nest-core does not do

| Feature | Where it actually lives |
|---|---|
| Desktop window / Tauri bridge | `nest-tauri` |
| UI components | `nest-tauri` + `ui/` (React) or `nest-gui` (egui) |
| Background tasks, Tokio, cancellation | `nest-task` (contracts) + `nest-task-runtime` (Tokio impl) |
| HTTP client / server | `nest-http-client` / `nest-http-serve` |
| Dynamic plugin loading | Not implemented — `Plugin` in nest-core is static, compile-time registration only |
| Trait-object service lookup (`dyn Repository`) | Not in v1 — lookup is always by concrete type |
| Structured error construction helpers beyond the base type | `nest-error` (re-exported by nest-core as `NestError`/`NestErrorKind`/`NestResult`) |

## Where this fits in the Nest layering

```text
Apps          separate repos     shipping products
  ▼
Modules       modules/crates/    optional integrations (nest-airtable, nest-tmdb, nest-claude, …)
  ▼
Core          core/crates/       framework hosts and contracts — nest-core is the bottom of this stack
```

Every module and host crate (`nest-app`, `nest-cli`, `nest-tui`, `nest-tauri`,
and every crate under `modules/crates/`) implements `Module` and depends on
`nest-core` for `AppBuilder`/`Module`/`Service`/`Lifecycle`. If you're adding
a new Nest module, its `module.rs` should look like the smallest existing
example (`nest-http-client`) before anything more elaborate.

## Related Nest docs

- `docs/nest-core/README.md`, `overview.md`, `getting-started.md`,
  `modules.md`, `services.md`, `application.md` — full prose documentation
- `docs/architecture.md` — the core/modules/apps layering and dependency rules
- `core/crates/nest-core/src/*.rs` — source of truth; prefer it over any
  stale prose doc if they disagree (the `api-reference.md` doc's `Module` and
  `NestError` shapes are out of date relative to source as of this writing —
  always verify against `module.rs`/`builder.rs` directly).
