//! HTTP server builder and lifecycle.

use std::net::SocketAddr;
use std::time::Duration;

use axum::extract::Request;
use axum::routing::{get, post, MethodRouter};
use axum::Router;
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

use nest_http::HttpMethod;

use crate::codes::NEST_HTTP_SERVE_CONFIG;
use crate::context::{extract_params, RequestContext};
use crate::cors::CorsConfig;
use crate::error::ServeError;
use crate::middleware::{wrap_with_middleware, MiddlewareLayer};
use crate::response::into_axum_response;
use crate::router::{nest_pattern_to_axum, RouteGroup, RouteRegistry};
use crate::spa::SpaConfig;
use crate::static_files::StaticFilesConfig;

/// Running HTTP server configuration.
pub struct HttpServer {
    builder: HttpServerBuilder,
}

/// Fluent builder for the HTTP host.
#[derive(Clone)]
pub struct HttpServerBuilder {
    name: String,
    bind_addr: SocketAddr,
    registry: RouteRegistry,
    middleware: Vec<MiddlewareLayer>,
    cors: Option<CorsConfig>,
    static_files: Vec<(String, StaticFilesConfig)>,
    spa: Option<SpaConfig>,
    shutdown_timeout: Duration,
}

/// Handle for a background test server.
pub struct TestServer {
    base_url: String,
    shutdown: Option<tokio::sync::oneshot::Sender<()>>,
}

impl HttpServer {
    /// Creates a new server builder.
    pub fn builder() -> HttpServerBuilder {
        HttpServerBuilder::new()
    }
}

impl HttpServerBuilder {
    /// Creates a default builder bound to `127.0.0.1:3000`.
    pub fn new() -> Self {
        Self {
            name: "nest-http-serve".to_string(),
            bind_addr: "127.0.0.1:3000".parse().expect("valid default bind address"),
            registry: RouteRegistry::new(),
            middleware: Vec::new(),
            cors: Some(CorsConfig::permissive()),
            static_files: Vec::new(),
            spa: None,
            shutdown_timeout: Duration::from_secs(30),
        }
    }

    /// Sets the server name used in logs.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Sets the listen address.
    pub fn bind(mut self, address: &str) -> Self {
        self.bind_addr = address
            .parse()
            .unwrap_or_else(|_| panic!("invalid bind address: {address}"));
        self
    }

    /// Adds routes from a route group.
    pub fn routes(mut self, group: RouteGroup) -> Self {
        self.registry.add_group(group);
        self
    }

    /// Alias for [`Self::routes`].
    pub fn route_group(self, group: RouteGroup) -> Self {
        self.routes(group)
    }

    /// Adds global middleware.
    pub fn middleware<M>(mut self, middleware: M) -> Self
    where
        M: crate::middleware::IntoMiddleware,
    {
        self.middleware.push(middleware.into_middleware());
        self
    }

    /// Configures CORS.
    pub fn cors(mut self, cors: CorsConfig) -> Self {
        self.cors = Some(cors);
        self
    }

    /// Serves static files under a URL prefix.
    pub fn serve_static(
        mut self,
        url_prefix: impl Into<String>,
        config: StaticFilesConfig,
    ) -> Self {
        self.static_files.push((url_prefix.into(), config));
        self
    }

    /// Serves a single-page application from a dist directory.
    pub fn serve_spa(mut self, dist_dir: impl Into<std::path::PathBuf>) -> Self {
        self.spa = Some(SpaConfig::new(dist_dir));
        self
    }

    /// Sets the graceful shutdown timeout.
    pub fn shutdown_timeout(mut self, timeout: Duration) -> Self {
        self.shutdown_timeout = timeout;
        self
    }

    /// Builds and runs the server until a shutdown signal is received.
    pub async fn run(self) -> Result<(), ServeError> {
        let listener = TcpListener::bind(self.bind_addr)
            .await
            .map_err(|error| config_error(format!("failed to bind {}: {error}", self.bind_addr)))?;

        let app = self.build_router()?;
        tracing::info!(server = %self.name, addr = %listener.local_addr().unwrap_or(self.bind_addr), "nest-http-serve listening");

        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal(self.shutdown_timeout))
            .await
            .map_err(|error| config_error(format!("server failed: {error}")))?;

        tracing::info!(server = %self.name, "nest-http-serve stopped");
        Ok(())
    }

    /// Starts the server in the background for integration tests.
    pub async fn spawn(mut self) -> Result<TestServer, ServeError> {
        self.bind_addr = "127.0.0.1:0"
            .parse()
            .expect("valid test bind address");

        let listener = TcpListener::bind(self.bind_addr)
            .await
            .map_err(|error| config_error(format!("failed to bind test listener: {error}")))?;
        let local_addr = listener
            .local_addr()
            .map_err(|error| config_error(format!("failed to read local addr: {error}")))?;
        let base_url = format!("http://{local_addr}");
        let app = self.build_router()?;
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();

        tokio::spawn(async move {
            let _ = axum::serve(listener, app)
                .with_graceful_shutdown(async {
                    let _ = shutdown_rx.await;
                })
                .await;
        });

        Ok(TestServer {
            base_url,
            shutdown: Some(shutdown_tx),
        })
    }

    fn build_router(&self) -> Result<Router, ServeError> {
        let mut router = Router::new();

        for route in self.registry.sorted_routes() {
            let handler = wrap_with_middleware(route.handler.clone(), &self.middleware);
            let method_router = method_router_for(route.method, route.pattern.clone(), handler);
            let axum_path = nest_pattern_to_axum(&route.pattern);
            router = router.route(&axum_path, method_router);
        }

        for (prefix, config) in &self.static_files {
            let service = ServeDir::new(config.root());
            router = router.nest_service(prefix, service);
        }

        if let Some(spa) = &self.spa {
            if !spa.dist_dir().is_dir() {
                return Err(config_error(format!(
                    "SPA dist directory not found: {}",
                    spa.dist_dir().display()
                )));
            }

            let index_path = spa.index_file().to_path_buf();
            let spa_service = ServeDir::new(spa.dist_dir())
                .not_found_service(ServeFile::new(index_path));
            router = router.fallback_service(spa_service);
        }

        if let Some(cors) = &self.cors {
            router = router.layer(cors.clone().into_layer());
        }

        router = router.layer(TraceLayer::new_for_http());
        Ok(router)
    }
}

impl TestServer {
    /// Returns the base URL for the running test server.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Stops the background server.
    pub async fn shutdown(mut self) {
        if let Some(tx) = self.shutdown.take() {
            let _ = tx.send(());
        }
    }
}

fn method_router_for(
    method: HttpMethod,
    pattern: String,
    handler: crate::router::Handler,
) -> MethodRouter {
    let dispatch = move |request: Request| {
        let pattern = pattern.clone();
        let handler = handler.clone();
        async move {
            let (parts, body) = request.into_parts();
            let body = match axum::body::to_bytes(body, usize::MAX).await {
                Ok(body) => body,
                Err(error) => {
                    return ServeError::json_invalid(error.to_string()).into_axum_response();
                }
            };

            let params = extract_params(&pattern, parts.uri.path());
            let ctx = RequestContext::from_parts(parts.method, &parts.uri, &parts.headers, params, body);
            into_axum_response(handler(ctx).await)
        }
    };

    match method {
        HttpMethod::Get => get(dispatch),
        HttpMethod::Post => post(dispatch),
        _ => get(dispatch),
    }
}

async fn shutdown_signal(timeout: Duration) {
    let ctrl_c = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::error!(%error, "failed to install Ctrl+C handler");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};

        match signal(SignalKind::terminate()) {
            Ok(mut stream) => {
                stream.recv().await;
            }
            Err(error) => {
                tracing::error!(%error, "failed to install SIGTERM handler");
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    tracing::info!(?timeout, "nest-http-serve shutdown signal received");
    tokio::time::sleep(timeout).await;
}

fn config_error(message: impl Into<String>) -> ServeError {
    ServeError::from(
        nest_http::HttpError::config(message).with_code(NEST_HTTP_SERVE_CONFIG),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::response::Json;
    use serde::Serialize;

    #[derive(Serialize)]
    struct Health {
        ok: bool,
    }

    async fn health(_ctx: RequestContext) -> HttpResult {
        Ok(Json(Health { ok: true }).into_response()?)
    }

    #[tokio::test]
    async fn health_route_returns_json() {
        let server = HttpServer::builder()
            .routes(RouteGroup::new("/api").get("/health", health))
            .spawn()
            .await
            .unwrap();

        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/api/health", server.base_url()))
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let body: serde_json::Value = response.json().await.unwrap();
        assert_eq!(body["ok"], true);

        server.shutdown().await;
    }

    #[tokio::test]
    async fn spa_fallback_serves_index_html() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::write(temp.path().join("index.html"), "<html>spa</html>").unwrap();

        let server = HttpServer::builder()
            .routes(RouteGroup::new("/api").get("/health", health))
            .serve_spa(temp.path())
            .spawn()
            .await
            .unwrap();

        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/movies/alien", server.base_url()))
            .send()
            .await
            .unwrap();
        assert!(response.status().is_success());
        let body = response.text().await.unwrap();
        assert!(body.contains("spa"));

        server.shutdown().await;
    }

    #[tokio::test]
    async fn static_route_wins_over_param_route() {
        async fn recent(_ctx: RequestContext) -> HttpResult {
            Ok(Json(serde_json::json!({ "route": "recent" })).into_response()?)
        }

        async fn by_slug(ctx: RequestContext) -> HttpResult {
            Ok(Json(serde_json::json!({ "route": "slug", "slug": ctx.param("slug")? }))
                .into_response()?)
        }

        let server = HttpServer::builder()
            .routes(
                RouteGroup::new("/movies")
                    .get("/recent", recent)
                    .get("/:slug", by_slug),
            )
            .spawn()
            .await
            .unwrap();

        let client = reqwest::Client::new();
        let recent = client
            .get(format!("{}/movies/recent", server.base_url()))
            .send()
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap();
        assert_eq!(recent["route"], "recent");

        let slug = client
            .get(format!("{}/movies/alien", server.base_url()))
            .send()
            .await
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .unwrap();
        assert_eq!(slug["route"], "slug");
        assert_eq!(slug["slug"], "alien");

        server.shutdown().await;
    }
}
