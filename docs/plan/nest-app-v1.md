# nest-app v1 Implementation Plan

## Status: Implemented

See [nest-app docs](../nest-app/README.md).

## Context

`nest-app` is the host-agnostic application container layered between `nest-core` and presentation hosts (`nest-cli`, `nest-tui`, `nest-tauri`, future `nest-http-server`). It wraps `nest_core::AppBuilder` with metadata, bootstrap validation, and lifecycle orchestration.

**Core rule:** `nest-core` defines building blocks. `nest-app` builds the standard container. Hosts decide how it is presented and executed.

**Desktop host:** [`nest-tauri`](./nest-tauri-v1.md) + React + Tailwind `ui/`. `nest-app` is **not** a GUI host.

## Crate boundaries

| Crate | Role |
|-------|------|
| `nest-core` | `AppBuilder`, `BuiltApp`, `AppContext`, `Module`, service registry |
| `nest-app` | `NestAppBuilder`, `NestApp`, `AppMetadata`, `AppBootstrapper`, `AppLifecycleRunner` |
| Hosts | CLI parsing, TUI/GUI loops, logging init, config file loading |

`nest-app` depends on `nest-core`, `nest-error`, and `tracing` only.

## Public API

- `NestApp::builder(name)` — facade over `AppBuilder`
- `NestApp` — built container with `metadata()`, `context()`, `startup()`, `shutdown()`
- `AppMetadata` / `AppEnvironment` — name, version, environment
- `AppBootstrapper` — app-level validation (non-empty name)
- `AppLifecycleRunner` — traced startup/shutdown with double-startup guard
- `HostApp` — host handoff trait implemented for `NestApp`

## Host handoff (v1)

Hosts accept an optional pre-built container:

```rust
let nest_app = NestApp::builder("kiwi")
    .register_service(ConfigService::new(loaded))?
    .module(FileModule::default())
    .build()?;

GuiApp::from_nest_app(nest_app)
    .view(MainView)
    .run();
```

When `from_nest_app` is used, the host skips rebuilding modules from `GuiApp::module()`. The host still owns logging init and startup-option parsing. If `ConfigService` is absent, the host falls back to `ConfigLoader`.

## Error codes

| Code | When |
|------|------|
| `NEST_APP_ALREADY_STARTED` | `startup()` called twice |
| `NEST_APP_NOT_STARTED` | Reserved for premature shutdown helpers |

## Deferred

- `ConfigModule` auto-loader (`nest-config` follow-up)
- Logging/config inside `nest-app` (stays in hosts)
- `nest-http-server` host
- Plugin auto-loading (`nest-plugins`)
- Full host API break (`GuiApp::new(app)` only) — v2 after migration
