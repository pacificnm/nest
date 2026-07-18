---
name: nest-logging
description: |-
  Optional tracing‑based logging infrastructure for Nest host applications.
  Provides configuration, initialization, and integration with `nest-error`.
sdk:
  version: 0.1.0
---
# nest‑logging Skill

## Overview

`nest-logging` is a crate that installs and configures a global `tracing` subscriber for Nest host applications (Kiwi, Nest CLI, etc.). It is **not** required by feature crates; those only depend on `tracing` and `nest-error`.

See the [Quick‑start](./README.md#quick-start) section in *docs/nest-logging/README.md* for an example of how to initialize logging.

## Configuration & Init

```rust
use nest_logging::prelude::*;

init(
    LoggingConfig::new("kiwi")
        .with_console()
        .with_file("./logs")
        .with_default_level(LogLevel::Info)
        .with_module_level("nest_data", LogLevel::Debug)
        .with_rotation(RotationPolicy::Daily)
        .with_retention(RetentionPolicy::Days(14))
        .capture_panics(true),
)?;
```

For full builder options, see the [Configuration](./documentation/../nest-logging/configuration.md) page.

## Logging Targets & Formats

Targets: `Console`, `File`, `JsonFile`. Formats: `Pretty`, `Compact`, `Json`.
See [Log formats](./documentation/../nest-logging/configuration.md#log-formats).

## Module Filtering

Per‑target log level control via `tracing_subscriber::EnvFilter`. Detail in the [Module filtering](./documentation/../nest-logging/module-filtering.md) page.

## Error Logging Integration

`nest-logging` implements `log_error` and `log_result` helpers that emit structured tracing events for `nest-error` errors. See [Error logging](./documentation/../nest-logging/error-logging.md).

---
*References*
- Overview: `overview.md`
- Configuration: `configuration.md`
- Module filtering: `module-filtering.md`
- Error logging: `error-logging.md`