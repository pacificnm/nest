# nest-logging v1 Implementation Plan

## Status: Implemented

See [nest-logging docs](../nest-logging/README.md).

## Context

Optional host-app logging on the tracing stack. `nest-core` and `nest-error` have no logging dependency. Feature crates use `tracing` only.

## Stack

- `tracing` — instrumentation
- `tracing-subscriber` — fmt, json, EnvFilter
- `tracing-appender` — non-blocking file writers, daily/hourly rotation

## Public API

- `LoggingConfig` builder
- `init()` / `init_logging()` → `LoggingGuard`
- `log_error(&NestError)` / `log_result`
- `cleanup_logs()` for retention
- Panic hook when `capture_panics(true)`

## Module filtering

Per-target levels via `EnvFilter`; `RUST_LOG` overrides when set.

## v1 limitations

- No size-based rotation
- Retention at init only
- No UI log sink
- No custom filter engine

## Follow-up

- Size rotation, periodic retention, Kiwi debug panel sink, TOML config
