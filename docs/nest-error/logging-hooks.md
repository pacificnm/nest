# Logging integration hooks

nest-error exposes a stable contract that **`nest-logging` implements**. nest-error has no dependency on tracing or nest-logging.

## Rule

- **nest-error** owns error shape and reporting
- **nest-logging** owns where/how errors are emitted

## Accessors (nest-error)

```rust
impl NestError {
    pub fn kind(&self) -> NestErrorKind;
    pub fn message(&self) -> &str;
    pub fn code(&self) -> Option<&str>;
    pub fn module(&self) -> Option<&str>;
    pub fn operation(&self) -> Option<&str>;
    pub fn help(&self) -> Option<&str>;
    pub fn source(&self) -> Option<&(dyn std::error::Error + 'static)>;
    pub fn report(&self) -> NestErrorReport;
    pub fn fields(&self) -> NestErrorFields<'_>;
}
```

## nest-logging implementation

Use `log_error` from `nest-logging`:

```rust
use nest_logging::log_error;

log_error(&error);
```

This emits a structured `tracing::error!` event with all nest-error metadata fields.

See [nest-logging error logging](../nest-logging/error-logging.md) for details.

## Feature crate guidance

- Return `NestResult<T>` from APIs
- Optionally use `tracing` for diagnostics
- Do **not** depend on `nest-logging`
- Let the host app install logging and call `log_error` at boundaries
