# Overview

## Purpose

`nest-error` is the shared error language for all Nest crates. It is not just an error enum — it is a consistent model for:

| Audience | What they get |
|----------|----------------|
| Developer | Source chain, debug metadata, stable codes |
| User | Message and recovery hints (`help`) |
| CLI | `NestErrorReport`, optional miette diagnostics |
| UI | `NestErrorReport` for dialogs and toasts |
| Logging | `fields()` accessors for structured events (via future `nest-logging`) |

## Design: struct + kind

`NestError` is a **struct**, not an enum. `NestErrorKind` classifies the error. Every module attaches the same metadata:

```text
kind       Validation
message    "Invalid email address"
code       NEST_VALIDATION_EMAIL
module     nest-forms
operation  save_customer
help       Enter a valid email like name@example.com
source     original lower-level error
```

This is more useful for UI and logging than a flat enum or `anyhow::Error`.

## NestErrorKind

```rust
pub enum NestErrorKind {
    Config, Io, Validation, Data, Command, Service, Module,
    Plugin, Task, Ui, Auth, Network, Unknown,
}
```

Each kind has a `label()` (e.g. `"service"`) and `title()` (e.g. `"Service Error"`) for reports and CLI output.

## Stable codes

Core codes live in `nest_error::codes`:

| Constant | Use |
|----------|-----|
| `NEST_SERVICE_NOT_FOUND` | Service registry lookup miss |
| `NEST_SERVICE_ALREADY_REGISTERED` | Duplicate service registration |
| `NEST_MODULE_CONFIG_FAILED` | Module configuration failure |
| `NEST_LIFECYCLE_FAILED` | Lifecycle hook failure |
| `NEST_UNKNOWN` | Uncategorized errors |

Modules should define their own `NEST_*` codes via `.with_code(...)`.

## Context chaining

Use `NestResultExt` to wrap lower-level errors without anyhow:

```rust
std::fs::read_to_string(path)
    .nest_context(NestErrorKind::Config, "Failed to read config")?;
```

## UI reports

```rust
let report = error.report();
// report.title, report.message, report.help, report.details
```

## Optional features

| Feature | Enables |
|---------|---------|
| `serde` | Serialize `NestError` metadata (source omitted) |
| `diagnostics` | miette `Diagnostic` impl and `diagnostic_report()` |

## Boundaries

**nest-error owns:** error shape, codes, reports, context helpers.

**nest-error does not own:** tracing emission, file logging, panic hooks — that is `nest-logging` (planned).

See [Logging hooks](logging-hooks.md) for the integration contract.
