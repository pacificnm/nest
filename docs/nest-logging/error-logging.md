# Error logging

`nest-logging` implements the contract defined in [nest-error logging hooks](../nest-error/logging-hooks.md).

## log_error

```rust
use nest_error::NestError;
use nest_logging::log_error;

let error = NestError::validation("Email is required")
    .with_code("NEST_VALIDATION_REQUIRED")
    .with_module("nest-forms")
    .with_help("Enter a valid email.");

log_error(&error);
```

Emits:

```text
tracing::error!(
    target: "nest_error",
    kind = ?error.kind(),
    code = error.code(),
    module = error.module(),
    operation = error.operation(),
    help = error.help(),
    error = %error,
    "Nest error occurred"
);
```

## log_result

```rust
use nest_logging::log_result;

let result: NestResult<()> = Err(error);
log_result(&result);  // logs only on Err
```

## Requirements

- A tracing subscriber must be installed (via `init()` or test setup)
- Feature crates should **not** call `log_error` directly unless they are the host app; prefer returning `NestResult` and let the host log failures

## Boundary

| Responsibility | Owner |
|----------------|-------|
| Error shape, codes, metadata | `nest-error` |
| Emitting structured tracing events | `nest-logging` |
