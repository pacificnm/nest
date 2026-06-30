# nest-logging Documentation

`nest-logging` is optional tracing-based logging infrastructure for Nest **host applications** (Kiwi, Nest CLI, etc.). Feature crates use `tracing` and `nest-error` only — they do not depend on this crate.

**Crate path:** [`crates/nest-logging`](../../crates/nest-logging)

## Quick start

```rust
use nest_logging::prelude::*;

init(
    LoggingConfig::new("kiwi")
        .with_console()
        .with_file("./logs")
        .with_default_level(LogLevel::Info)
        .with_module_level("nest_data", LogLevel::Debug)
        .with_module_level("kiwi::git", LogLevel::Trace)
        .with_rotation(RotationPolicy::Daily)
        .with_retention(RetentionPolicy::Days(14))
        .capture_panics(true),
)?;

tracing::info!(target: "nest_data", "query executed");
```

## Documentation

| Document | Description |
|----------|-------------|
| [Overview](overview.md) | Boundaries, dependency rules, tracing conventions |
| [Configuration](configuration.md) | Targets, formats, rotation, retention |
| [Module filtering](module-filtering.md) | EnvFilter, per-target levels, `RUST_LOG` |
| [Error logging](error-logging.md) | `log_error` and nest-error integration |
| [API reference](api-reference.md) | Public API summary |

## Related

- [Implementation plan](../plan/nest-logging-v1.md)
- [nest-error logging hooks](../nest-error/logging-hooks.md)
