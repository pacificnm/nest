# nest-http v1 Implementation Plan

## Status: Implemented

See [nest-http docs](../nest-http/README.md) and [nest-http-client docs](../nest-http-client/README.md).

## Context

Shared HTTP contracts (`nest-http`) and async reqwest client (`nest-http-client`). Host owns the Tokio runtime; modules provide futures only. `nest-http-server` deferred until nest-serve/nest-api-serve.

## Crate boundaries

| Crate | Role |
|-------|------|
| `nest-http` | `HttpError`, methods, status, headers, auth/retry traits, `ApiResponse` |
| `nest-http-client` | `HttpClientService`, `HttpClientModule`, reqwest adapter, tracing |
| `nest-core` | Registers service; never `.await`s |

## nest-http

- No reqwest, axum, tokio, tracing, or nest-core
- `BearerTokenAuth`, `FixedRetryPolicy` built-ins
- `HttpError` with `nest_code()` for conversion

## nest-http-client

- Async-only: `get_json`, `post_json`, `send`
- `http_error_to_nest_error()` (orphan-rule safe; no `From` impl)
- `tracing` for request start/complete/fail/retry
- No dependency on `nest-logging`
- `ExampleApiClient` demonstrates typed API wrappers

## v1 limitations

- No `nest-http-server`
- No embedded Tokio runtime
- No OAuth refresh

## Follow-up

- ~~`nest-http-server` + axum for nest-serve~~ → [`nest-http-serve`](../nest-http-serve/README.md) (v0.1)
- `AsyncRuntimeService` in nest-tasks
