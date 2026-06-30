# API reference

**Crate:** `nest_error`  
**Version:** 0.1.0

## Prelude

```rust
use nest_error::prelude::*;
// NestError, NestErrorKind, NestErrorReport, NestResult, NestResultExt, codes
```

## Types

| Type | Description |
|------|-------------|
| `NestResult<T>` | `Result<T, NestError>` |
| `NestError` | Structured error struct |
| `NestErrorKind` | Error category enum |
| `NestErrorReport` | UI/CLI display snapshot |
| `NestErrorFields<'a>` | Logging adapter snapshot |

## NestError constructors

| Method | Kind | Default code |
|--------|------|--------------|
| `new(kind, message)` | any | — |
| `config(message)` | Config | — |
| `io(message)` | Io | — |
| `validation(message)` | Validation | — |
| `data(message)` | Data | — |
| `command(message)` | Command | — |
| `service(message)` | Service | — |
| `module_error(message)` | Module | `NEST_MODULE_CONFIG_FAILED` |
| `plugin(message)` | Plugin | — |
| `task(message)` | Task | — |
| `ui(message)` | Ui | — |
| `auth(message)` | Auth | — |
| `network(message)` | Network | — |
| `unknown(message)` | Unknown | `NEST_UNKNOWN` |
| `service_not_found(type_name)` | Service | `NEST_SERVICE_NOT_FOUND` |
| `service_already_registered(type_name)` | Service | `NEST_SERVICE_ALREADY_REGISTERED` |
| `lifecycle(message)` | Module | `NEST_LIFECYCLE_FAILED` |

## Builder methods

| Method | Description |
|--------|-------------|
| `with_code` | Stable error code |
| `with_module` | Originating Nest module |
| `with_operation` | Failed operation name |
| `with_help` | User recovery hint |
| `with_source` | Attach source error chain |

## NestResultExt

```rust
fn nest_context(self, kind: NestErrorKind, message: impl Into<String>) -> NestResult<T>;
fn nest_context_with(self, builder: impl FnOnce(NestError) -> NestError) -> NestResult<T>;
```

Implemented for `Result<T, E>` where `E: Error + Send + Sync + 'static`.

## Features

| Feature | Description |
|---------|-------------|
| `serde` | Serialize metadata (no source chain) |
| `diagnostics` | miette `Diagnostic` + `diagnostic_report()` |

## nest-core re-exports

```rust
pub use nest_error::{
    NestError, NestErrorFields, NestErrorKind, NestErrorReport,
    NestResult, NestResultExt,
};
```
