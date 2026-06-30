# Errors

nest-core does **not** define its own error types. It re-exports the shared error model from [`nest-error`](../nest-error/README.md).

```rust
pub use nest_error::{
    NestError, NestErrorFields, NestErrorKind, NestErrorReport,
    NestResult, NestResultExt,
};
```

Use `nest_core::{NestError, NestResult}` as before — the implementation lives in `nest-error`.

## NestResult

```rust
pub type NestResult<T> = Result<T, NestError>;
```

## NestError (struct)

`NestError` is a struct with `NestErrorKind`, message, optional code/module/operation/help, and an optional source chain. See [nest-error overview](../nest-error/overview.md) for the full model.

## nest-core service errors

nest-core produces these errors from the service registry:

| Situation | Constructor | Code |
|-----------|-------------|------|
| Service not registered | `NestError::service_not_found(type_name)` | `NEST_SERVICE_NOT_FOUND` |
| Duplicate registration | `NestError::service_already_registered(type_name)` | `NEST_SERVICE_ALREADY_REGISTERED` |

### Example: service lookup

```rust
let err = ctx.service::<MissingService>().unwrap_err();
assert_eq!(err.kind(), NestErrorKind::Service);
assert_eq!(err.code(), Some(nest_error::codes::NEST_SERVICE_NOT_FOUND));
```

### Example: duplicate registration

```rust
app.register_service(Logger::new())?;
app.register_service(Logger::new())?; // Service + NEST_SERVICE_ALREADY_REGISTERED
```

## Module and lifecycle errors

Modules should use `NestError` builders with appropriate kinds and codes:

```rust
NestError::module_error("failed to load plugin")
    .with_module("nest-plugins")

NestError::lifecycle("startup hook failed")
    .with_operation("on_startup")
```

## Error propagation

```rust
let git = ctx.service::<GitService>()?;  // propagates NestError

let mut built = app.build()?;
built.startup()?;
built.shutdown()?;
```

## Testing

Match on kind and code instead of enum variants:

```rust
use nest_core::{NestError, NestErrorKind};
use nest_error::codes;

let err = registry.get::<Logger>().unwrap_err();
assert_eq!(err.kind(), NestErrorKind::Service);
assert_eq!(err.code(), Some(codes::NEST_SERVICE_NOT_FOUND));
```

## Further reading

- [nest-error documentation](../nest-error/README.md)
- [Logging integration hooks](../nest-error/logging-hooks.md)
