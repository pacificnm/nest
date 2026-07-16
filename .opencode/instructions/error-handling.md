# Error handling

Use `NestError` / `NestResult` (from `nest-error`) for all fallible operations
in Nest code — don't roll a custom error enum per crate.

```rust
use nest_error::prelude::*;

fn load_config(path: &str) -> NestResult<String> {
    std::fs::read_to_string(path)
        .nest_context(NestErrorKind::Config, "Failed to read application config")
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

- Attach a stable `NEST_*` code with `.with_code(...)` for anything a caller
  might branch on or a user might search for.
- Attach `.with_module(...)` (the crate name) and `.with_help(...)` (a
  recovery hint) wherever the error can reasonably suggest one.
- Wrap lower-level errors (`std::io::Error`, third-party crate errors) via
  `NestResultExt::nest_context` (or `nest_context_with` for custom
  builder logic), not ad-hoc `.map_err` boilerplate.
- `nest-error` owns error *shape*; it does not log. `nest-logging` records
  errors via `error.kind()`, `error.code()`, `error.fields()` — see
  [logging.md](logging.md).

## `clippy::result_large_err`

`NestError` is ~144 bytes (message, kind, and several `Option<String>`
fields). Any public function returning `NestResult<T>` will trip
`clippy::result_large_err` under `-D warnings` unless the crate has
`#![allow(clippy::result_large_err)]` in `lib.rs` (see `nest-claude`,
`nest-task` for the established pattern). Check whether that allow already
exists before adding a new crate that returns `NestResult` from a public API
— this is easy to miss and only surfaces at `cargo clippy -- -D warnings`
time, not at `cargo build`.

See [docs/nest-error/README.md](../../docs/nest-error/README.md).
