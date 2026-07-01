//! Route registry and route groups.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use nest_http::HttpMethod;

use crate::context::RequestContext;
use crate::middleware::{wrap_with_middleware, MiddlewareLayer};
use crate::response::HttpResult;

/// Type-erased async route handler.
pub type Handler = Arc<
    dyn Fn(RequestContext) -> Pin<Box<dyn Future<Output = HttpResult> + Send>> + Send + Sync,
>;

/// A single route definition.
#[derive(Clone)]
pub struct RouteDefinition {
    /// HTTP method.
    pub method: HttpMethod,
    /// Full path pattern (e.g. `/api/movies/:slug`).
    pub pattern: String,
    /// Route handler.
    pub handler: Handler,
}

/// Collects routes from one or more groups.
#[derive(Default)]
pub struct RouteRegistry {
    routes: Vec<RouteDefinition>,
}

impl RouteRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds all routes from a group.
    pub fn add_group(&mut self, group: RouteGroup) {
        self.routes.extend(group.into_routes());
    }

    /// Returns routes sorted for match priority: static, param, wildcard.
    pub fn sorted_routes(&self) -> Vec<RouteDefinition> {
        let mut routes = self.routes.clone();
        routes.sort_by(|left, right| route_priority(&left.pattern).cmp(&route_priority(&right.pattern)));
        routes
    }

    /// Returns all routes.
    pub fn routes(&self) -> &[RouteDefinition] {
        &self.routes
    }
}

/// A prefixed group of routes.
#[derive(Default)]
pub struct RouteGroup {
    prefix: String,
    routes: Vec<RouteDefinition>,
    middleware: Vec<MiddlewareLayer>,
}

impl RouteGroup {
    /// Creates a route group with the given URL prefix.
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
            routes: Vec::new(),
            middleware: Vec::new(),
        }
    }

    /// Registers a GET route.
    pub fn get<H>(mut self, path: impl Into<String>, handler: H) -> Self
    where
        H: IntoHandler,
    {
        self.routes.push(RouteDefinition {
            method: HttpMethod::Get,
            pattern: join_paths(&self.prefix, path.into()),
            handler: handler.into_handler(),
        });
        self
    }

    /// Registers a POST route.
    pub fn post<H>(mut self, path: impl Into<String>, handler: H) -> Self
    where
        H: IntoHandler,
    {
        self.routes.push(RouteDefinition {
            method: HttpMethod::Post,
            pattern: join_paths(&self.prefix, path.into()),
            handler: handler.into_handler(),
        });
        self
    }

    /// Adds group-level middleware applied to routes in this group.
    pub fn middleware<M>(mut self, middleware: M) -> Self
    where
        M: IntoMiddleware,
    {
        self.middleware.push(middleware.into_middleware());
        self
    }

    /// Consumes the group and returns route definitions with middleware applied.
    pub(crate) fn into_routes(self) -> Vec<RouteDefinition> {
        self.routes
            .into_iter()
            .map(|mut route| {
                route.handler = wrap_with_middleware(route.handler, &self.middleware);
                route
            })
            .collect()
    }
}

/// Converts async handler functions into route handlers.
pub trait IntoHandler {
    /// Converts into a type-erased handler.
    fn into_handler(self) -> Handler;
}

impl<F, Fut> IntoHandler for F
where
    F: Fn(RequestContext) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = HttpResult> + Send + 'static,
{
    fn into_handler(self) -> Handler {
        Arc::new(move |ctx| Box::pin(self(ctx)))
    }
}

fn join_paths(prefix: &str, path: String) -> String {
    let prefix = prefix.trim_end_matches('/');
    let path = path.trim_start_matches('/');
    if prefix.is_empty() {
        if path.is_empty() {
            "/".to_string()
        } else {
            format!("/{path}")
        }
    } else if path.is_empty() {
        prefix.to_string()
    } else {
        format!("{prefix}/{path}")
    }
}

/// Returns route priority (lower sorts first): static, param, wildcard.
fn route_priority(pattern: &str) -> (u8, std::cmp::Reverse<usize>) {
    if pattern.split('/').any(|segment| segment.starts_with('*')) {
        (2, std::cmp::Reverse(pattern.len()))
    } else if pattern.split('/').any(|segment| segment.starts_with(':')) {
        (1, std::cmp::Reverse(pattern.len()))
    } else {
        (0, std::cmp::Reverse(pattern.len()))
    }
}

/// Converts a Nest path pattern to an axum path pattern.
pub(crate) fn nest_pattern_to_axum(pattern: &str) -> String {
    if pattern == "/" {
        return "/".to_string();
    }

    let mut result = String::new();
    for segment in pattern.split('/').filter(|segment| !segment.is_empty()) {
        result.push('/');
        if let Some(name) = segment.strip_prefix(':') {
            result.push('{');
            result.push_str(name);
            result.push('}');
        } else if let Some(name) = segment.strip_prefix('*') {
            result.push_str("{*");
            result.push_str(name);
            result.push('}');
        } else {
            result.push_str(segment);
        }
    }

    if result.is_empty() {
        "/".to_string()
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::response::Json;

    #[test]
    fn converts_param_pattern() {
        assert_eq!(nest_pattern_to_axum("/movies/:slug"), "/movies/{slug}");
    }

    #[test]
    fn converts_wildcard_pattern() {
        assert_eq!(nest_pattern_to_axum("/files/*path"), "/files/{*path}");
    }

    #[test]
    fn static_routes_sort_before_params() {
        let mut registry = RouteRegistry::new();
        registry.add_group(
            RouteGroup::new("/movies")
                .get("/:slug", |_| async { Ok(Json(1).into()) })
                .get("/recent", |_| async { Ok(Json(2).into()) }),
        );

        let patterns: Vec<_> = registry
            .sorted_routes()
            .into_iter()
            .map(|route| route.pattern)
            .collect();
        assert_eq!(patterns, vec!["/movies/recent", "/movies/:slug"]);
    }
}
