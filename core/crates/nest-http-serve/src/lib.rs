//! Reusable HTTP host for serving Nest applications.
//!
//! Apps define routes; `nest-http-serve` provides the HTTP server lifecycle,
//! routing, middleware hooks, JSON responses, static files, and SPA fallback.
//!
//! ## Example
//!
//! ```no_run
//! use nest_http_serve::{HttpServer, RouteGroup, RequestContext, HttpResult, Json};
//!
//! async fn health(_ctx: RequestContext) -> HttpResult {
//!     Ok(Json(serde_json::json!({ "ok": true })).into_response()?)
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<(), nest_http_serve::ServeError> {
//!     HttpServer::builder()
//!         .name("my-app")
//!         .bind("0.0.0.0:3000")
//!         .routes(RouteGroup::new("/api").get("/health", health))
//!         .serve_spa("./web/dist")
//!         .run()
//!         .await
//! }
//! ```

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

mod codes;
mod context;
mod cors;
mod error;
mod middleware;
mod prelude;
mod response;
mod router;
mod server;
mod spa;
mod static_files;

pub use context::RequestContext;
pub use cors::CorsConfig;
pub use error::{http_error_to_nest_error, ServeError};
pub use middleware::Next;
pub use response::{HttpResponse, HttpResult, Json};
pub use router::{RouteDefinition, RouteGroup, RouteRegistry};
pub use server::{HttpServer, HttpServerBuilder, TestServer};
pub use spa::SpaConfig;
pub use static_files::StaticFilesConfig;

pub use nest_error::{NestError, NestResult};
pub use nest_http::{HttpError, HttpMethod, HttpStatus};

pub mod prelude {
    //! Common imports for nest-http-serve handlers.
    pub use crate::prelude::*;
}
