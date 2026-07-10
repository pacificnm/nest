//! CORS configuration.

use std::time::Duration;

use tower_http::cors::{Any, CorsLayer};

/// CORS settings for the HTTP server.
#[derive(Debug, Clone)]
pub struct CorsConfig {
    allow_any_origin: bool,
    allowed_origins: Vec<String>,
    allowed_methods: Vec<String>,
    allowed_headers: Vec<String>,
    max_age: Option<Duration>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allow_any_origin: true,
            allowed_origins: Vec::new(),
            allowed_methods: vec![
                "GET".to_string(),
                "POST".to_string(),
                "PUT".to_string(),
                "PATCH".to_string(),
                "DELETE".to_string(),
                "OPTIONS".to_string(),
            ],
            allowed_headers: vec!["content-type".to_string(), "authorization".to_string()],
            max_age: Some(Duration::from_secs(3600)),
        }
    }
}

impl CorsConfig {
    /// Creates a permissive development CORS configuration.
    pub fn permissive() -> Self {
        Self::default()
    }

    /// Restricts allowed origins to the given list.
    pub fn allow_origins(mut self, origins: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.allow_any_origin = false;
        self.allowed_origins = origins.into_iter().map(Into::into).collect();
        self
    }

    /// Builds a tower CORS layer.
    pub(crate) fn into_layer(self) -> CorsLayer {
        let mut layer = CorsLayer::new();

        if self.allow_any_origin {
            layer = layer.allow_origin(Any);
        } else {
            let origins: Vec<_> = self
                .allowed_origins
                .iter()
                .filter_map(|origin| origin.parse().ok())
                .collect();
            layer = layer.allow_origin(origins);
        }

        let methods: Vec<_> = self
            .allowed_methods
            .iter()
            .filter_map(|method| method.parse().ok())
            .collect();
        layer = layer.allow_methods(methods);

        let headers: Vec<_> = self
            .allowed_headers
            .iter()
            .filter_map(|header| header.parse().ok())
            .collect();
        layer = layer.allow_headers(headers);

        if let Some(max_age) = self.max_age {
            layer = layer.max_age(max_age);
        }

        layer
    }
}
