# nest-http-serve

Reusable HTTP host for serving Nest applications.

**Crate path:** [`core/crates/nest-http-serve`](../../core/crates/nest-http-serve)

## Role

`nest-http-serve` is the HTTP **host** crate. Apps define routes and handlers; this crate provides server lifecycle, routing, middleware hooks, JSON responses, static files, SPA fallback, CORS, and graceful shutdown.

It does **not** know about any specific app (Loon, Kiwi, Finch, …).

## Quick start

```rust
use nest_http_serve::{HttpServer, RouteGroup, RequestContext, HttpResult, Json};

async fn health(_ctx: RequestContext) -> HttpResult {
    Ok(Json(serde_json::json!({ "ok": true })).into_response()?)
}

#[tokio::main]
async fn main() -> Result<(), nest_http_serve::ServeError> {
    HttpServer::builder()
        .name("my-app")
        .bind("0.0.0.0:3000")
        .routes(RouteGroup::new("/api").get("/health", health))
        .serve_spa("./web/dist")
        .run()
        .await
}
```

## Routes in the app

```rust
fn api_routes() -> RouteGroup {
    RouteGroup::new("/api")
        .get("/health", health)
        .get("/movies", list_movies)
        .get("/movies/:slug", get_movie)
}
```

```rust
async fn get_movie(ctx: RequestContext) -> HttpResult {
    let slug = ctx.param("slug")?;
    Ok(Json(serde_json::json!({ "slug": slug })).into_response()?)
}
```

## Route matching

Paths support `:param` and `*wildcard` segments. Static routes win over parameterized routes, which win over wildcards.

## v0.1 scope

| Included | Deferred |
|----------|----------|
| GET / POST | PUT / PATCH / DELETE |
| JSON body + responses | OAuth / sessions |
| Path + query params | WebSockets / SSE |
| Route groups | OpenAPI |
| Static files + SPA fallback | Rate limiting |
| Middleware hooks | File streaming |
| CORS + trace logging | Multipart upload |
| Graceful shutdown | `nest-app` injection |

## Dependency rule

```text
nest-error  ←  nest-http  ←  nest-http-client
                    ↑
                    │
              nest-http-serve
```

`nest-http-serve` does not depend on `nest-core`, modules, or apps.

## Related

- [Implementation plan](../plan/nest-http-serve-v1.md)
- [nest-http](../nest-http/README.md) — shared contracts
- [nest-http-client](../nest-http-client/README.md) — outbound HTTP
