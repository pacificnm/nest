# nest-http

Shared HTTP contracts for the [Nest framework](../../README.md).

**Crate path:** [`core/crates/nest-http`](../../core/crates/nest-http)

## Role

nest-http defines the **shared HTTP language**. It does not perform networking — see `nest-http-client` (consume) and future `nest-http-server` (serve).

| Type | Purpose |
|------|---------|
| `HttpMethod` / `HttpStatus` | Request method and response status |
| `HeaderMap` | Header helpers |
| `HttpRequest` / `HttpResponse` | Lightweight request/response models |
| `ApiResponse<T>` | Standard API envelope (serde feature) |
| `Page` / `PageRequest` | Pagination |
| `RequestId` / `CorrelationId` | Tracing and correlation |
| `TimeoutConfig` | Connect and request timeouts |
| `AuthStrategy` | Pluggable auth (e.g. `BearerTokenAuth`) |
| `RetryPolicy` | Pluggable retry (e.g. `FixedRetryPolicy`) |
| `HttpError` | Structured HTTP errors |

## Features

```toml
nest-http = { path = "../nest-http", features = ["serde"] }
```

Enable `serde` for `ApiResponse`, pagination, and JSON metadata fields.

## Dependency rule

```
nest-error  ←  nest-http  ←  nest-http-client
```

nest-http does **not** depend on nest-core, reqwest, or Tokio.

## Related

- [nest-http-client](../nest-http-client/README.md) — async reqwest client
