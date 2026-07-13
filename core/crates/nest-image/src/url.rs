//! Resolve API-relative image paths against a server base URL.

#![allow(clippy::result_large_err)]

use nest_error::{NestError, NestResult};

/// Joins `server_url` and a relative or absolute image path.
pub fn resolve_url(server_url: &str, path: &str) -> NestResult<String> {
    let server_url = server_url.trim().trim_end_matches('/');
    let path = path.trim();

    if server_url.is_empty() {
        return Err(NestError::validation("server_url is empty"));
    }
    if path.is_empty() {
        return Err(NestError::validation("image path is empty"));
    }

    if path.starts_with("http://") || path.starts_with("https://") {
        return Ok(path.to_string());
    }

    let suffix = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    };

    Ok(format!("{server_url}{suffix}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_artwork_proxy_path() {
        let url = resolve_url(
            "http://192.168.88.205:3000",
            "/api/artwork/alien-1979/poster",
        )
        .unwrap();
        assert_eq!(
            url,
            "http://192.168.88.205:3000/api/artwork/alien-1979/poster"
        );
    }

    #[test]
    fn accepts_absolute_url() {
        let url = resolve_url("http://127.0.0.1:3000", "https://cdn.example/p.jpg").unwrap();
        assert_eq!(url, "https://cdn.example/p.jpg");
    }
}
