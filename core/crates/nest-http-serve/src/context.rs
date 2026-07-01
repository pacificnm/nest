//! Per-request context for handlers.

use std::collections::HashMap;

use axum::body::Bytes;
use axum::http::{HeaderMap as AxumHeaderMap, Method, Uri};
use serde::de::DeserializeOwned;

use nest_http::{HeaderMap, HttpMethod};

use crate::error::ServeError;

/// Per-request data passed to route handlers.
#[derive(Debug, Clone)]
pub struct RequestContext {
    method: HttpMethod,
    path: String,
    query: HashMap<String, String>,
    headers: HeaderMap,
    params: HashMap<String, String>,
    body: Bytes,
}

impl RequestContext {
    /// Creates a request context from HTTP parts.
    pub fn new(
        method: HttpMethod,
        path: impl Into<String>,
        query: HashMap<String, String>,
        headers: HeaderMap,
        params: HashMap<String, String>,
        body: Bytes,
    ) -> Self {
        Self {
            method,
            path: path.into(),
            query,
            headers,
            params,
            body,
        }
    }

    /// Returns the HTTP method.
    pub fn method(&self) -> HttpMethod {
        self.method
    }

    /// Returns the request path.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Returns a path parameter.
    pub fn param(&self, name: &str) -> Result<&str, ServeError> {
        self.params
            .get(name)
            .map(String::as_str)
            .ok_or_else(|| ServeError::param_missing(name))
    }

    /// Returns an optional query parameter.
    pub fn query(&self, name: &str) -> Option<&str> {
        self.query.get(name).map(String::as_str)
    }

    /// Returns a required query parameter.
    pub fn query_required(&self, name: &str) -> Result<&str, ServeError> {
        self.query
            .get(name)
            .map(String::as_str)
            .ok_or_else(|| ServeError::query_missing(name))
    }

    /// Returns a request header value.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers.get(name)
    }

    /// Returns the raw request body.
    pub fn body(&self) -> &[u8] {
        &self.body
    }

    /// Deserializes the request body as JSON.
    pub fn json<T: DeserializeOwned>(&self) -> Result<T, ServeError> {
        if self.body.is_empty() {
            return Err(ServeError::json_invalid("request body is empty"));
        }
        serde_json::from_slice(&self.body).map_err(|error| ServeError::json_invalid(error))
    }

    /// Builds a context from axum request parts.
    pub(crate) fn from_parts(
        method: Method,
        uri: &Uri,
        headers: &AxumHeaderMap,
        params: HashMap<String, String>,
        body: Bytes,
    ) -> Self {
        Self {
            method: axum_method_to_http(method),
            path: uri.path().to_string(),
            query: parse_query(uri.query()),
            headers: axum_headers_to_nest(headers),
            params,
            body,
        }
    }
}

fn axum_method_to_http(method: Method) -> HttpMethod {
    match method {
        Method::GET => HttpMethod::Get,
        Method::POST => HttpMethod::Post,
        Method::PUT => HttpMethod::Put,
        Method::PATCH => HttpMethod::Patch,
        Method::DELETE => HttpMethod::Delete,
        Method::HEAD => HttpMethod::Head,
        Method::OPTIONS => HttpMethod::Options,
        _ => HttpMethod::Get,
    }
}

fn axum_headers_to_nest(headers: &AxumHeaderMap) -> HeaderMap {
    let mut map = HeaderMap::new();
    for (name, value) in headers.iter() {
        if let Ok(value) = value.to_str() {
            map.insert(name.as_str(), value);
        }
    }
    map
}

pub(crate) fn parse_query(query: Option<&str>) -> HashMap<String, String> {
    let Some(query) = query else {
        return HashMap::new();
    };

    query
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next()?;
            let value = parts.next().unwrap_or("");
            Some((key.to_string(), value.to_string()))
        })
        .collect()
}

/// Extracts path parameters from a matched pattern and request path.
pub(crate) fn extract_params(pattern: &str, path: &str) -> HashMap<String, String> {
    let mut params = HashMap::new();
    let pattern_segments: Vec<&str> = pattern.split('/').filter(|segment| !segment.is_empty()).collect();
    let path_segments: Vec<&str> = path.split('/').filter(|segment| !segment.is_empty()).collect();

    let mut pattern_index = 0;
    let mut path_index = 0;

    while pattern_index < pattern_segments.len() {
        let segment = pattern_segments[pattern_index];
        if let Some(name) = segment.strip_prefix(':') {
            if path_index >= path_segments.len() {
                break;
            }
            params.insert(name.to_string(), path_segments[path_index].to_string());
            pattern_index += 1;
            path_index += 1;
        } else if let Some(name) = segment.strip_prefix('*') {
            let rest = path_segments[path_index..].join("/");
            params.insert(name.to_string(), rest);
            break;
        } else {
            if path_index >= path_segments.len() || segment != path_segments[path_index] {
                break;
            }
            pattern_index += 1;
            path_index += 1;
        }
    }

    params
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_slug_param() {
        let params = extract_params("/movies/:slug", "/movies/alien");
        assert_eq!(params.get("slug").map(String::as_str), Some("alien"));
    }

    #[test]
    fn static_path_does_not_capture_recent_as_slug() {
        let params = extract_params("/movies/:slug", "/movies/recent");
        assert_eq!(params.get("slug").map(String::as_str), Some("recent"));
    }

    #[test]
    fn wildcard_param() {
        let params = extract_params("/files/*path", "/files/a/b/c");
        assert_eq!(params.get("path").map(String::as_str), Some("a/b/c"));
    }

    #[test]
    fn parse_query_string() {
        let query = parse_query(Some("page=2&sort=title"));
        assert_eq!(query.get("page").map(String::as_str), Some("2"));
        assert_eq!(query.get("sort").map(String::as_str), Some("title"));
    }
}
