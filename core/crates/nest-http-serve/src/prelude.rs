//! Common nest-http-serve imports.

pub use crate::context::RequestContext;
pub use crate::cors::CorsConfig;
pub use crate::error::ServeError;
pub use crate::middleware::Next;
pub use crate::response::{HttpResponse, HttpResult, Json};
pub use crate::router::{RouteGroup, RouteRegistry};
pub use crate::server::{HttpServer, HttpServerBuilder};
pub use crate::spa::SpaConfig;
pub use crate::static_files::StaticFilesConfig;

pub use nest_error::{NestError, NestResult};
pub use nest_http::{HttpError, HttpMethod, HttpStatus};
