# nest-error Documentation

`nest-error` is the shared error foundation for the [Nest framework](../../README.md). Every Nest crate returns `NestResult<T>` and uses the structured [`NestError`](overview.md) type.

**Crate path:** [`crates/nest-error`](../../crates/nest-error)

## Quick start

```rust
use nest_error::prelude::*;

fn load_config(path: &str) -> NestResult<String> {
    std::fs::read_to_string(path).nest_context(
        NestErrorKind::Config,
        "Failed to read application config",
    )
}

fn validate_email(email: &str) -> NestResult<()> {
    if email.is_empty() {
        return Err(
            NestError::validation("Email is required")
                .with_code("NEST_VALIDATION_REQUIRED")
                .with_module("nest-forms")
                .with_help("Enter an email before saving."),
        );
    }
    Ok(())
}
```

## Documentation

| Document | Description |
|----------|-------------|
| [Overview](overview.md) | Design, audiences, struct + kind model |
| [Logging hooks](logging-hooks.md) | Contract for future `nest-logging` integration |
| [API reference](api-reference.md) | Public API summary |

## Using from nest-core

`nest-core` re-exports nest-error types. Existing code can keep:

```rust
use nest_core::{NestError, NestErrorKind, NestResult};
```

New crates should prefer:

```rust
use nest_error::prelude::*;
```

## Related

- [Implementation plan](../plan/nest-error-v1.md)
- [nest-core errors](../nest-core/errors.md) — re-export and service-specific errors
