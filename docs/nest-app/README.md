# nest-app

Host-agnostic application container for the [Nest framework](../../README.md).

**Crate path:** [`crates/nest-app`](../../crates/nest-app)

`nest-app` wraps [`nest_core::AppBuilder`](../../crates/nest-core/src/builder.rs) with metadata and lifecycle orchestration. Host crates execute the container; they do not replace it.

## Quick start

```rust
use nest_app::{AppEnvironment, NestApp};
use nest_config::{ConfigLoader, ConfigService};
use nest_file::FileModule;
use nest_gui::{GuiApp, GuiView};

struct MainView;

impl GuiView for MainView {
    fn ui(&mut self, ui: &mut egui::Ui, _ctx: &nest_core::AppContext) -> nest_error::NestResult<()> {
        ui.heading("Hello");
        Ok(())
    }
}

fn main() {
    let loaded = ConfigLoader::file_or_search("kiwi", None).load().unwrap();
    let nest_app = NestApp::builder("kiwi")
        .version(env!("CARGO_PKG_VERSION"))
        .environment(AppEnvironment::Development)
        .register_service(ConfigService::new(loaded))
        .unwrap()
        .module(FileModule::default())
        .build()
        .unwrap();

    GuiApp::from_nest_app(nest_app)
        .view(MainView)
        .run();
}
```

## Builder API

`NestAppBuilder` forwards to `nest_core::AppBuilder`:

| Method | Purpose |
|--------|---------|
| `version` / `environment` | Set `AppMetadata` |
| `module` | Register a `Module` |
| `register_service` | Register a singleton service |
| `register_lifecycle` | Register startup/shutdown hooks |
| `register_panel` / `register_command` / `register_job` / `register_plugin` | Extension-point introspection |
| `build` | Validate metadata and return `NestApp` |

```rust
let app = NestApp::builder("kiwi")
    .version("1.0.0")
    .environment(AppEnvironment::Production)
    .module(MyModule)
    .register_service(MyService::new())?
    .register_lifecycle(MyLifecycle)
    .build()?;
```

## Container lifecycle

```rust
let mut app = NestApp::builder("kiwi").build()?;

app.startup()?;   // runs core lifecycle hooks; traced as app.startup
// host main loop
app.shutdown()?;  // idempotent when not started
```

`AppLifecycleRunner::startup` / `shutdown` are also available for static dispatch.

Calling `startup()` twice returns `NEST_APP_ALREADY_STARTED`.

## Host handoff

The `HostApp` trait lets hosts consume a pre-built container:

```rust
pub trait HostApp {
    fn metadata(&self) -> &AppMetadata;
    fn context(&self) -> &AppContext;
    fn into_nest_app(self) -> NestApp;
}
```

Each host exposes `from_nest_app` / `with_nest_app`:

| Host | When container is attached |
|------|----------------------------|
| `nest-cli` | Skips `AppBuilder` module loop; uses container context for dispatch |
| `nest-tui` | Skips module registration in `prepare_runtime` |
| `nest-gui` | Same as TUI |

**Service injection constraint:** register `ConfigService` and other services **before** `build()`. Hosts still own config file discovery when no `ConfigService` is present.

## Responsibility table

| Concern | Owner |
|---------|-------|
| Module dependency validation | `nest-core` (`configure_modules`) |
| App name validation | `nest-app` (`AppBootstrapper`) |
| Startup/shutdown tracing | `nest-app` (`AppLifecycleRunner`) |
| CLI / TUI / GUI presentation | Host crates |
| Logging initialization | Host crates |
| Config file loading | Host crates (or pre-register `ConfigService`) |

## Related docs

- [nest-core application](../nest-core/application.md)
- [nest-cli](../nest-cli/README.md) · [nest-tui](../nest-tui/README.md) · [nest-gui](../nest-gui/README.md)
- [Implementation plan](../plan/nest-app-v1.md)
