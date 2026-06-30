# nest-core Documentation

`nest-core` is the foundation crate of the [Nest framework](../../README.md). It defines the minimum application architecture: module configuration, explicit singleton service registration, typed service lookup, synchronous lifecycle hooks, and extension-point traits for optional crates.

**Crate path:** [`core/crates/nest-core`](../../core/crates/nest-core)

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

## Documentation

| Document | Description |
|----------|-------------|
| [Overview](overview.md) | Role of nest-core, design principles, boundaries |
| [Getting started](getting-started.md) | Step-by-step guide to building an app with nest-core |
| [Services](services.md) | Service registry, registration rules, lookup |
| [Application & lifecycle](application.md) | `AppBuilder`, `AppContext`, `BuiltApp`, lifecycle hooks |
| [Modules & plugins](modules.md) | `Module` and `Plugin` patterns |
| [Extension traits](extension-traits.md) | `Panel`, `Command`, `Job`, `RegistrationInfo` |
| [Errors](errors.md) | `NestError` and error handling |
| [API reference](api-reference.md) | Consolidated public API |

## Design principle

Every optional capability (themes, validation, logging, plugins, tasks, etc.) integrates with nest-core through the same **Module + Service Registry** pattern. Once you understand `ThemeModule` or `ValidationModule`, the rest of the framework follows the same model.

## Related

- [nest-core v1 implementation plan](../plan/nest-core-v1.md)
- [nest-validation](../nest-validation/README.md) — optional validation module
- [nest-data](../nest-data/README.md) — optional data layer module
- [nest-file](../nest-file/README.md) — sync file I/O service
- [nest-task](../nest-task/README.md) — task execution contracts
- [nest-task-runtime](../nest-task-runtime/README.md) — Tokio task manager module
- [nest-http](../nest-http/README.md) — shared HTTP contracts
- [nest-http-client](../nest-http-client/README.md) — async HTTP client module
- [nest-theme](../nest-theme/README.md) — optional theme module
- [nest-error](../nest-error/README.md) — shared error model (re-exported by nest-core)
- [Framework vision (README)](../../README.md)
