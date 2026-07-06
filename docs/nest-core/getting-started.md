# Getting started

This guide walks through building a minimal application using nest-core directly. In production, a **host crate** wraps this flow: [`nest-cli`](../nest-cli/README.md), [`nest-tui`](../nest-tui/README.md), or [`nest-tauri`](../nest-tauri/README.md) + React `ui/` for desktop.

## Add the dependency

```toml
[dependencies]
nest-core = { path = "../core/crates/nest-core" }
```

## 1. Define services

A service is any type that is `Send + Sync + 'static`. nest-core implements the `Service` marker trait automatically for qualifying types.

```rust
struct Logger {
    prefix: String,
}

impl Logger {
    fn log(&self, message: &str) {
        eprintln!("[{}] {}", self.prefix, message);
    }
}

struct Settings {
    theme: String,
}
```

## 2. Create a module

Modules configure the application by registering services and other capabilities on the `AppBuilder`.

```rust
use nest_core::{AppBuilder, Module};

struct CoreModule;

impl Module for CoreModule {
    fn configure(&self, app: &mut AppBuilder) {
        app.register_service(Logger {
            prefix: "my-app".to_string(),
        })
        .expect("logger");

        app.register_service(Settings {
            theme: "dark".to_string(),
        })
        .expect("settings");
    }
}
```

`AppBuilder::module` calls `configure` immediately when the module is added. Module order determines registration order.

## 3. Register lifecycle hooks (optional)

Lifecycle handlers run after `build()` and before the application exits.

```rust
use std::sync::Arc;

use nest_core::{AppContext, Lifecycle, NestResult};

struct AppLifecycle;

impl Lifecycle for AppLifecycle {
    fn on_startup(&mut self, ctx: Arc<AppContext>) -> NestResult<()> {
        let logger = ctx.service::<Logger>()?;
        logger.log("Application starting");
        Ok(())
    }

    fn on_shutdown(&mut self, ctx: Arc<AppContext>) -> NestResult<()> {
        let logger = ctx.service::<Logger>()?;
        logger.log("Application shutting down");
        Ok(())
    }
}
```

## 4. Build and run

```rust
use nest_core::AppBuilder;

let mut app = AppBuilder::new().module(CoreModule);
app.register_lifecycle(AppLifecycle);

let mut built = app.build().expect("build failed");
built.startup().expect("startup failed");

// At this point, use built.context for service lookup:
let settings = built.context.service::<Settings>().unwrap();
println!("Theme: {}", settings.theme);

built.shutdown().expect("shutdown failed");
```

## 5. Share context across components

`BuiltApp::context` is an `Arc<AppContext>`. Clone it to pass into UI callbacks, panels, or other long-lived components:

```rust
let ctx = built.context.clone();

// Later, in another component:
fn do_work(ctx: &AppContext) -> nest_core::NestResult<()> {
    let logger = ctx.service::<Logger>()?;
    logger.log("Working...");
    Ok(())
}
```

The registry is **frozen after `build()`**. Services cannot be registered on `AppContext` at runtime.

## Common patterns

### Register services outside modules

You can register services directly on the builder, in addition to module configuration:

```rust
let mut app = AppBuilder::new().module(CoreModule);
app.register_service(ExtraService::new()).unwrap();
```

### Multiple modules

Modules are applied in the order they are added:

```rust
let app = AppBuilder::new()
    .module(LoggingModule)
    .module(GitModule)
    .module(SettingsModule);
```

### Plugin-style registration

For larger feature bundles, implement `Plugin`:

```rust
use nest_core::Plugin;

struct GitPlugin;

impl Plugin for GitPlugin {
    fn register(&self, app: &mut AppBuilder) {
        app.register_service(GitService::new()).unwrap();
        app.register_command(OpenRepository).unwrap();
    }
}

let mut app = AppBuilder::new();
app.register_plugin(GitPlugin);
```

## Next steps

- [Services](services.md) — registry rules and constraints
- [Application & lifecycle](application.md) — full `AppBuilder` API
- [Modules & plugins](modules.md) — organizing features into modules
