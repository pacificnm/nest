# nest-http-client

Async HTTP client for consuming external APIs in the Nest framework.

**Crate path:** [`core/crates/nest-http-client`](../../core/crates/nest-http-client)

## Quick start

```rust
use nest_core::AppBuilder;
use nest_http_client::{HttpClientModule, HttpClientService};

#[tokio::main]
async fn main() -> nest_error::NestResult<()> {
    let built = AppBuilder::new()
        .module(HttpClientModule::default())
        .build()?;

    let http = built.context.service::<HttpClientService>()?;
    let customer: Customer = http
        .get_json("https://api.example.com/customers/123")
        .await?;
    Ok(())
}
```

## Runtime requirement

nest-http-client provides async APIs only — it never calls `Runtime::new()` or `block_on`.

| Host | Pattern |
|------|---------|
| Nest-owned app | [`nest-task-runtime`](../nest-task-runtime/README.md) `TaskRuntimeModule::owned()` |
| Server / tests | `#[tokio::main]` or `TaskRuntimeModule::from_current()` |

For long-running HTTP work (large downloads, batch API calls), wrap calls inside [`TaskManager::spawn`](../nest-task/README.md) rather than blocking the UI thread.

## Configuration

```rust
use std::sync::Arc;
use nest_http::{BearerTokenAuth, FixedRetryPolicy, TimeoutConfig};
use nest_http_client::{HttpClientConfig, HttpClientModule};
use std::time::Duration;

HttpClientModule::with_config(
    HttpClientConfig::default()
        .with_auth(Arc::new(BearerTokenAuth::new("token")))
        .with_retry(FixedRetryPolicy::new(3, Duration::from_millis(200)))
        .with_timeout(TimeoutConfig::new(
            Duration::from_secs(5),
            Duration::from_secs(30),
        )),
)
```

## Typed API clients

Wrap `HttpClientService` in domain-specific clients:

```rust
pub struct ExampleApiClient {
    http: HttpClientService,
    base_url: String,
}

impl ExampleApiClient {
    pub async fn get_customer(&self, id: &str) -> NestResult<Customer> {
        self.http
            .get_json(&format!("{}/customers/{id}", self.base_url))
            .await
    }
}
```

See [`ExampleApiClient`](../../core/crates/nest-http-client/src/example.rs) in the crate.

## Logging and errors

- Emits `tracing` events (start, complete, fail, retry) — no `nest-logging` dependency
- Converts `HttpError` → `NestError` via `http_error_to_nest_error()`
- Uses `NestErrorKind::Network` with codes like `NEST_HTTP_TIMEOUT`

## Module integration

```rust
pub const HTTP_CLIENT_MODULE_ID: ModuleId = ModuleId("nest-http-client");
```

Registered via `HttpClientModule`; lookup with `ctx.service::<HttpClientService>()?`.

## Related

- [nest-http](../nest-http/README.md) — shared contracts
- [nest-logging](../nest-logging/overview.md) — optional host subscriber
