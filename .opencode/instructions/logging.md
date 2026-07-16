# Logging

`nest-logging` installs and configures a global `tracing` subscriber. It is
only for **host applications** (a CLI binary, a Tauri app, a server) — it is
**not** required by `nest-core`, `nest-error`, or feature/module crates.

| Crate | Logging dependency |
|-------|---------------------|
| `nest-core` | None |
| `nest-error` | None (defines error shape only) |
| Feature/module crates (`nest-data`, `nest-git`, …) | `tracing` macros only |
| Host app (CLI, Tauri, server) | `nest-logging`, calls `init()` once at startup |

Feature crates must not depend on `nest-logging` directly — emit `tracing`
events with an explicit target instead:

```rust
tracing::info!(target: "nest_data", "query executed");
tracing::warn!(target: "nest_plugins", "plugin failed to load");
```

Use the crate or logical module path as the target so `EnvFilter` per-target
levels and `RUST_LOG` apply correctly.

## Host app setup

```rust
use nest_logging::prelude::*;

init(
    LoggingConfig::new("my-app")
        .with_console()
        .with_file("./logs")
        .with_default_level(LogLevel::Info)
        .with_module_level("nest_data", LogLevel::Debug)
        .with_rotation(RotationPolicy::Daily)
        .with_retention(RetentionPolicy::Days(14))
        .capture_panics(true),
)?;
```

`nest-cli` hosts typically call this automatically via `CliApp::with_logging`
during the shared bootstrap sequence — check whether the host already wires
it before calling `init()` a second time.

See [docs/nest-logging/](../../docs/nest-logging/README.md) — overview,
configuration, module filtering, and `nest-error` integration.
