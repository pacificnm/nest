# Module filtering

In v1, **module filtering** means per-target log level control using `tracing_subscriber::EnvFilter` — the same model as `RUST_LOG`.

## Config → directive string

```rust
LoggingConfig::new("kiwi")
    .with_default_level(LogLevel::Info)
    .with_module_level("nest_core", LogLevel::Warn)
    .with_module_level("nest_data", LogLevel::Debug)
    .with_module_level("kiwi::git", LogLevel::Trace)
```

Produces:

```text
info,nest_core=warn,nest_data=debug,kiwi::git=trace
```

## Environment override

When `env_override` is true (default) and `RUST_LOG` is set:

```bash
RUST_LOG="info,nest_data=debug,kiwi::git=trace" ./my-app
```

The environment variable takes precedence over configured module levels.

## Target naming convention

Use explicit `target:` in tracing macros:

```rust
tracing::info!(target: "nest_data", "query executed");
```

Targets should match the module level keys in configuration.

## What v1 does not filter

- Plugin instances
- Tenants or workspaces
- Users or feature flags
- Dynamic UI filter persistence

Those require a custom filter engine in a future version.
