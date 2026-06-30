# Overview

## Role

`nest-logging` installs and configures a global `tracing` subscriber for Nest host applications. It is **not** required by `nest-core`, `nest-error`, or feature crates.

## Rules

| Crate | Logging dependency |
|-------|---------------------|
| `nest-core` | None |
| `nest-error` | None (defines error shape only) |
| `nest-git`, `nest-data`, etc. | `tracing` macros only |
| Host app (Kiwi, CLI) | `nest-logging` calls `init()` |

**nest-error creates structured errors. nest-logging records them.**

## Technology stack

| Component | Crate |
|-----------|-------|
| Instrumentation | `tracing` |
| Subscriber / formatting / filtering | `tracing-subscriber` |
| File writers / rotation | `tracing-appender` |

## Tracing conventions

Feature crates emit events with explicit targets:

```rust
tracing::info!(target: "nest_data", "query executed");
tracing::debug!(target: "kiwi::git", "status parsed");
tracing::warn!(target: "nest_plugins", "plugin failed to load");
```

Use crate or logical module paths as targets so `EnvFilter` module levels apply.

## What host apps get

- Console logging (pretty, compact, or JSON)
- Rolling text and/or JSON log files
- Per-target log levels via `EnvFilter`
- `RUST_LOG` environment override
- Log retention cleanup at init
- Panic hook that logs via tracing
- `log_error(&NestError)` helper

## What feature crates should not do

- Depend on `nest-logging`
- Call `init()` themselves
- Assume a subscriber is installed (tracing no-ops safely if not)

## v1 limitations

- Size-based rotation (`RotationPolicy::SizeBytes`) is not implemented
- No UI log panel sink (Kiwi debug console — later)
- No custom filter engine (tenant, workspace, plugin instance)
- Retention runs at init only (periodic sweep — v1.1)
