//! Authentication strategy contracts.

use crate::error::HttpResult;
use crate::request::HttpRequest;

/// Applies authentication credentials to outgoing requests.
pub trait AuthStrategy: Send + Sync {
    /// Mutates the request with auth headers or parameters.
    fn apply(&self, request: &mut HttpRequest) -> HttpResult<()>;
}

/// Bearer token authentication.
#[derive(Debug, Clone)]
pub struct BearerTokenAuth {
    token: String,
}

impl BearerTokenAuth {
    /// Creates a bearer token auth strategy.
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }
}

impl AuthStrategy for BearerTokenAuth {
    fn apply(&self, request: &mut HttpRequest) -> HttpResult<()> {
        request
            .headers
            .insert("authorization", format!("Bearer {}", self.token));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::method::HttpMethod;

    #[test]
    fn bearer_sets_authorization_header() {
        let auth = BearerTokenAuth::new("secret");
        let mut request = HttpRequest::new(HttpMethod::Get, "https://api.example.com");
        auth.apply(&mut request).unwrap();
        assert_eq!(
            request.headers.get("authorization"),
            Some("Bearer secret")
        );
    }
}
