# nest-http-serve v1 Implementation Plan

## Status: Implemented

See [nest-http-serve docs](../nest-http-serve/README.md).

Implements the deferred HTTP **host** role referenced as `nest-http-server` in [nest-http v1](nest-http-v1.md), [nest-app v1](nest-app-v1.md), and [nest-task v1](nest-task-v1.md).

## Naming

| Name | Use |
|------|-----|
| **Crate** | `nest-http-serve` |
| **Rust import** | `nest_http_serve` |
| **Docs path** | `docs/nest-http-serve/` (when implemented) |
| **Crate path** | `core/crates/nest-http-serve` |

Older docs say `nest-http-server`; that referred to this host crate. Update cross-references when v1 ships.

## Context

Nest already has:

- **`nest-http`** — shared HTTP contracts (methods, status, headers, `HttpError`, `ApiResponse`, …). No networking.
- **`nest-http-client`** — async reqwest client for **consuming** external APIs. Host owns Tokio; crate never blocks.

`nest-http-serve` completes the triangle: a reusable HTTP **host** for **serving** requests. Apps (Loon, Kiwi, Finch, …) define routes and handlers; the crate provides server lifecycle, routing, middleware hooks, JSON helpers, static files, and SPA fallback.

**Design principle:** Apps provide routes. `nest-http-serve` provides the host.

The crate must **not** know about Loon, Kiwi, Finch, media, files, OAuth, or any specific app.

## Purpose

`nest-http-serve` is the reusable HTTP host for Nest apps. v0.1 is intentionally small: enough to ship a JSON API + SPA shell for products like Loon without baking in product logic.

### In scope (host responsibilities)

| Area | v0.1 |
|------|------|
| HTTP server lifecycle | Yes |
| Routing + route groups | Yes |
| Dynamic path params | Yes |
| Middleware / hooks | Yes (minimal) |
| JSON responses | Yes |
| Static file serving | Yes |
| SPA fallback | Yes |
| Request context | Yes |
| Graceful shutdown | Yes |
| Trace logging | Yes |
| CORS config | Yes |

### Out of scope (later modules or features)

OAuth, sessions, WebSockets, SSE, OpenAPI, rate limiting, file streaming, multipart upload.

## Dependency direction

```text
apps/loon (or kiwi, finch, …)
        │
        ▼ depends on
nest-http-serve
        │
        ├── nest-http      (contracts — reuse HttpMethod, HttpStatus, HttpError, …)
        ├── nest-error     (NestError mapping for handlers)
        ├── axum           (internal adapter — not leaked in public API)
        ├── tokio          (runtime — host owns it)
        └── tower-http     (CORS, trace, static files)

nest-http-serve
        ✗ does not depend on apps/*
        ✗ does not depend on modules/*
        ✗ does not depend on nest-core / nest-app (v0.1 — see integration below)
```

```text
nest-error  ←  nest-http  ←  nest-http-client
                    ↑
                    │
              nest-http-serve
```

`nest-http-serve` and `nest-http-client` are siblings; neither depends on the other.

## Crate boundaries

| Crate | Role |
|-------|------|
| `nest-http` | Shared types: `HttpMethod`, `HttpStatus`, `HttpError`, `HeaderMap`, `ApiResponse`, … |
| `nest-http-serve` | `HttpServer`, routing, `RequestContext`, static/SPA, Axum adapter |
| `nest-http-client` | Outbound HTTP (unchanged) |
| Apps | Route definitions, domain handlers, product config |

## Responsibilities

```text
nest-http-serve
├── Server builder          HttpServer / HttpServerBuilder
├── Route registry          RouteRegistry, RouteDefinition
├── Route groups            RouteGroup
├── Request context         RequestContext (params, query, headers, body)
├── Response helpers        Json, HttpResponse, HttpResult
├── Error mapping           HttpError ↔ handler errors
├── Static file serving     StaticFilesConfig
├── SPA serving             SpaConfig (fallback to index.html)
├── Middleware hooks        Middleware, Next
└── Axum adapter            internal — maps Nest routes → axum Router
```

## Core types

### Reuse from `nest-http` (do not duplicate)

- `HttpMethod`
- `HttpStatus`
- `HttpError` / `HttpResult` (contract-level)
- `HeaderMap`
- `ApiResponse<T>` (when `serde` feature enabled)

### New in `nest-http-serve`

| Type | Responsibility |
|------|----------------|
| `HttpServer` | Running server handle; shutdown signal |
| `HttpServerBuilder` | Fluent configuration before `run()` |
| `RouteRegistry` | Collected routes from groups; match order |
| `RouteGroup` | Prefix + method handlers (`/api`, `/admin`, …) |
| `RouteDefinition` | Method, pattern, handler fn |
| `RequestContext` | Per-request: path params, query, headers, extensions |
| `HttpResponse` | Serve-layer response (may wrap `nest_http::HttpResponse`) |
| `HttpResult` | Handler result alias (`Result<impl IntoResponse, …>`) |
| `Middleware` | `async fn(RequestContext, Next) -> HttpResult` |
| `Next` | Invoke downstream middleware / handler |
| `StaticFilesConfig` | Root dir, cache headers, index file |
| `SpaConfig` | Dist dir + fallback route for client-side routing |

## Public API (target)

```rust
use nest_http_serve::{
    HttpServer,
    RouteGroup,
    RequestContext,
    HttpResult,
    Json,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    HttpServer::builder()
        .name("loon")
        .bind("0.0.0.0:3000")
        .routes(api_routes())
        .serve_spa("./web/dist")
        .run()
        .await
}
```

Routes live in the **app**, not in the host crate:

```rust
fn api_routes() -> RouteGroup {
    RouteGroup::new("/api")
        .get("/health", health)
        .get("/movies", list_movies)
        .get("/movies/:slug", get_movie)
}
```

Handler:

```rust
async fn get_movie(ctx: RequestContext) -> HttpResult {
    let slug = ctx.param("slug")?;

    Ok(Json(MovieResponse {
        slug: slug.to_string(),
        title: "Alien".to_string(),
    }).into())
}
```

### Builder options (v0.1)

| Method | Purpose |
|--------|---------|
| `name(&str)` | Server name for logs / `User-Agent` prefix |
| `bind(addr)` | Listen address |
| `routes(RouteGroup)` | Mount API routes (repeatable) |
| `route_group(RouteGroup)` | Alias / additional groups |
| `middleware(Middleware)` | Global middleware stack |
| `cors(CorsConfig)` | tower-http CORS layer |
| `serve_static(path, StaticFilesConfig)` | Static assets under a URL prefix |
| `serve_spa(dist_dir)` | SPA dist + fallback to `index.html` |
| `shutdown_timeout(Duration)` | Graceful shutdown window |
| `run()` | Build router, bind, serve until signal |

## Route matching

Support patterns:

```text
/movies/:slug
/movies/:slug/watch
/files/*path
```

**Match priority** (static wins over dynamic):

1. **Static routes** — exact path match (`/movies/recent`)
2. **Parameterized routes** — `:param` segments (`/movies/:slug`)
3. **Wildcard routes** — `*path` suffix catch-all (`/files/*path`)

Example: `/movies/recent` must match a static route when registered, not `:slug = "recent"`.

Implementation: register routes in priority buckets when building `RouteRegistry`; Axum router order reflects this (static literals before param routes before wildcards).

### Path params

- `ctx.param("slug")?` → `&str` or typed extractors in follow-up
- Missing param → `400` with `HttpError`

### Query params

- `ctx.query("page")` → optional
- `ctx.query_required("id")?` → required

## Middleware

v0.1 middleware is a simple async hook chain:

```rust
async fn request_id(ctx: RequestContext, next: Next) -> HttpResult {
    // read/set correlation id, call next.run(ctx).await
    next.run(ctx).await
}
```

Global middleware via `HttpServerBuilder::middleware`. Per-group middleware via `RouteGroup::middleware` (optional v0.1 stretch; document API even if implemented in v0.2).

Built-in tower layers (internal): trace, CORS. No auth middleware in v0.1.

## Static files and SPA

### Static files

```rust
.serve_static("/assets", StaticFilesConfig::new("./web/dist/assets"))
```

Uses `tower-http` `ServeDir` internally. v0.1: whole-file read only (no streaming — deferred).

### SPA fallback

```rust
.serve_spa("./web/dist")
```

Behavior:

1. Try static file from dist
2. If not found and path has no file extension → serve `index.html`
3. API routes under `/api` registered **before** SPA catch-all

Mount order: API routes → static prefixes → SPA fallback (`/*` or equivalent).

## Error mapping

Handlers return `HttpResult`. Errors map to HTTP responses consistently:

| Source | HTTP | Body |
|--------|------|------|
| `nest_http::HttpError` | `status()` | JSON `{ "error": { "code", "message" } }` |
| `nest_error::NestError` | By `NestErrorKind` | Same envelope |
| `ctx.param()` miss | `400 Bad Request` | Structured error |
| Unhandled panic | `500` | Generic message (no stack in body) |

Reuse `nest_http::HttpError::nest_code()` where applicable. Do **not** add `From` impls that violate orphan rules; use explicit converters like `nest-http-client`'s `http_error_to_nest_error()`.

## Logging and shutdown

- **Trace:** `tracing` events on request start, complete, fail (request id, method, path, status, duration). No `nest-logging` dependency — host app may install a subscriber separately.
- **Graceful shutdown:** listen for `SIGINT` / `SIGTERM` (and `ctrl_c` on dev); `graceful_shutdown` with configurable timeout; in-flight requests complete or timeout.

## Runtime integration

`nest-http-serve` **owns** the Tokio runtime in the default `run()` path (like a minimal host).

| Host pattern | When |
|--------------|------|
| `#[tokio::main]` + `HttpServer::builder().run()` | Standalone server binary (Loon v1) |
| `TaskRuntimeModule::from_current()` | Tests or embedding inside an existing runtime (future) |

Align with [nest-task-runtime](../nest-task-runtime/README.md): long-running work inside handlers should use `TaskManager::spawn`, not block the Axum worker thread.

### Optional `nest-app` integration (v0.2)

v0.1 does **not** require `nest-app`. Follow-up:

```rust
HttpServer::builder()
    .nest_app(built_nest_app)  // inject AppContext into RequestContext extensions
    .routes(api_routes())
    .run()
    .await
```

Handlers then call `ctx.service::<T>()?` like other Nest hosts.

## v0.1 scope checklist

### Ship in v0.1

- [x] `GET` and `POST` routes
- [x] JSON request body (POST) and JSON responses
- [x] Path params (`:name`) and query params
- [x] Route groups with prefix
- [x] Static route priority over params
- [x] Wildcard `*path` routes
- [x] Static file serving (non-streaming)
- [x] SPA fallback (`index.html`)
- [x] Global middleware hook (`Middleware` / `Next`)
- [x] CORS configuration
- [x] Trace logging (`tracing`)
- [x] Graceful shutdown
- [x] Axum adapter (internal)
- [x] Integration test: health route + SPA fallback

### Explicitly deferred

| Feature | Target |
|---------|--------|
| `PUT` / `PATCH` / `DELETE` | v0.2 |
| Per-route middleware | v0.2 |
| `nest-app` / `AppContext` injection | v0.2 |
| OAuth / sessions | separate module |
| WebSockets / SSE | separate module or v0.3 |
| OpenAPI / Swagger | separate module |
| Rate limiting | tower layer or module |
| File streaming / range requests | v0.2+ |
| Multipart upload | separate module |
| TLS termination | reverse proxy or later |

## Workspace layout

```text
core/crates/nest-http-serve/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── prelude.rs
    ├── server.rs           # HttpServer, HttpServerBuilder
    ├── router.rs           # RouteRegistry, RouteGroup, RouteDefinition
    ├── context.rs          # RequestContext
    ├── response.rs         # Json, HttpResponse, IntoResponse bridge
    ├── error.rs            # handler error mapping
    ├── middleware.rs       # Middleware, Next
    ├── static_files.rs     # StaticFilesConfig
    ├── spa.rs              # SpaConfig
    ├── cors.rs             # CorsConfig
    └── axum/               # internal adapter
        ├── mod.rs
        ├── router.rs
        └── handlers.rs
```

Root `Cargo.toml` additions:

```toml
# [workspace.members]
"core/crates/nest-http-serve",

# [workspace.dependencies]
nest-http-serve = { path = "core/crates/nest-http-serve" }
```

### `Cargo.toml` dependencies (draft)

```toml
[dependencies]
nest-http = { workspace = true, features = ["serde"] }
nest-error = { workspace = true }
axum = "0.8"
tokio = { version = "1", features = ["rt-multi-thread", "macros", "signal"] }
tower = "0.5"
tower-http = { version = "0.6", features = ["cors", "trace", "fs"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
http = "1"

[dev-dependencies]
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
reqwest = { version = "0.12", features = ["json"] }
tempfile = "3"
```

## Example app wiring (Loon)

Loon (in `apps/loon`, separate repo) depends on `nest-http-serve` only for the host:

```text
apps/loon
   ├── src/main.rs          # HttpServer::builder()…
   ├── src/routes/          # RouteGroup definitions
   └── web/dist/            # SPA build output
```

No Loon types inside `nest-http-serve`. No `nest-http-serve` imports from `apps/loon`.

## Testing strategy

| Test | Type |
|------|------|
| Route match order (static vs `:param` vs `*`) | Unit |
| `RequestContext` param/query parsing | Unit |
| Health `GET /api/health` → 200 JSON | Integration (in-process server) |
| SPA unknown path → `index.html` | Integration |
| CORS preflight | Integration |
| Graceful shutdown completes in-flight request | Integration |

## Follow-up

- `nest-http-serve` v0.2: `nest-app` handoff, `PUT`/`DELETE`, per-group middleware
- Update [nest-http README](../nest-http/README.md) dependency diagram
- Update [nest-http v1](nest-http-v1.md) follow-up section (mark server planned → implemented)
- `docs/nest-http-serve/README.md` when crate lands
- Re-index project memory: `./scripts/index-memory.sh`

## Related

- [nest-http v1](nest-http-v1.md) — contracts + client; server was deferred here
- [nest-http-client README](../nest-http-client/README.md) — outbound HTTP
- [nest-task-runtime README](../nest-task-runtime/README.md) — runtime ownership patterns
- [architecture.md](../architecture.md) — core host placement
