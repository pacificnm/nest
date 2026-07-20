//! Bridges an acquired [`Token`] into [`nest_http`]'s (synchronous)
//! [`AuthStrategy`], and gives whatever performs refreshes a handle to
//! update the credential this strategy is currently applying.
//!
//! `AuthStrategy::apply` is sync — it can't itself call the async
//! `TokenStore` or `OAuthClient::refresh`. So the acquired token lives in an
//! `Arc<RwLock<Token>>`: `apply` only ever takes a fast synchronous read of
//! the cache, while whoever owns [`OAuthTokenAuth::handle`] (typically the
//! same code driving login/refresh) writes the new token back after each
//! successful refresh.

use std::sync::{Arc, RwLock};

use nest_auth::Token;
use nest_http::{AuthStrategy, HttpRequest, HttpResult};

/// An [`AuthStrategy`] backed by a live, updatable [`Token`].
#[derive(Clone)]
pub struct OAuthTokenAuth {
    token: Arc<RwLock<Token>>,
}

impl OAuthTokenAuth {
    /// Wraps an initial token.
    pub fn new(token: Token) -> Self {
        Self {
            token: Arc::new(RwLock::new(token)),
        }
    }

    /// Returns a shared handle to the underlying token cache. Whoever
    /// refreshes the token (e.g. after [`crate::OAuthClient::refresh`]
    /// succeeds) should call [`Self::set_token`] on a clone of this same
    /// [`OAuthTokenAuth`], or write through this handle directly, so
    /// subsequent [`AuthStrategy::apply`] calls pick up the new token.
    pub fn handle(&self) -> Arc<RwLock<Token>> {
        self.token.clone()
    }

    /// Replaces the cached token (after a refresh or re-login).
    pub fn set_token(&self, token: Token) {
        *self.token.write().expect("oauth token cache lock poisoned") = token;
    }

    /// Returns a clone of the currently cached token.
    pub fn current_token(&self) -> Token {
        self.token
            .read()
            .expect("oauth token cache lock poisoned")
            .clone()
    }
}

impl AuthStrategy for OAuthTokenAuth {
    fn apply(&self, request: &mut HttpRequest) -> HttpResult<()> {
        let token = self.token.read().expect("oauth token cache lock poisoned");
        request
            .headers
            .insert("authorization", format!("Bearer {}", token.access_token));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nest_http::HttpMethod;

    #[test]
    fn apply_sets_bearer_header_from_the_cached_token() {
        let auth = OAuthTokenAuth::new(Token::new("initial-access-token"));
        let mut request = HttpRequest::new(HttpMethod::Get, "https://api.example.com");

        auth.apply(&mut request).expect("apply");

        assert_eq!(
            request.headers.get("authorization"),
            Some("Bearer initial-access-token")
        );
    }

    #[test]
    fn apply_picks_up_a_token_set_after_construction() {
        let auth = OAuthTokenAuth::new(Token::new("initial-access-token"));
        auth.set_token(Token::new("refreshed-access-token"));

        let mut request = HttpRequest::new(HttpMethod::Get, "https://api.example.com");
        auth.apply(&mut request).expect("apply");

        assert_eq!(
            request.headers.get("authorization"),
            Some("Bearer refreshed-access-token")
        );
    }

    #[test]
    fn handle_shares_state_with_the_original() {
        let auth = OAuthTokenAuth::new(Token::new("initial-access-token"));
        let handle = auth.handle();
        *handle.write().expect("lock") = Token::new("written-through-handle");

        assert_eq!(auth.current_token().access_token, "written-through-handle");
    }
}
