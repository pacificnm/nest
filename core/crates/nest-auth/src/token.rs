//! The token representation shared across auth mechanisms.

use serde::{Deserialize, Serialize};

/// An acquired credential — an OAuth access/refresh token pair, or any
/// similarly-shaped bearer credential.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Token {
    /// The bearer credential itself.
    pub access_token: String,
    /// Credential used to obtain a new `access_token` once this one
    /// expires, when the issuing mechanism supports refreshing (most
    /// OAuth2 flows do; Schwab's does, with a separate, shorter lifetime —
    /// see [`Token::needs_reauth_on_expiry`]).
    pub refresh_token: Option<String>,
    /// Unix epoch milliseconds this token stops being valid at, if the
    /// issuing mechanism reports an expiry — consistent with every other
    /// wire/storage timestamp convention across the framework (epoch
    /// millis, not a native timestamp type).
    pub expires_at_ms: Option<i64>,
    /// Space-delimited scope string, as returned by the issuing server, if any.
    pub scope: Option<String>,
}

impl Token {
    /// Creates a token with just an access token — no refresh token,
    /// expiry, or scope.
    pub fn new(access_token: impl Into<String>) -> Self {
        Self {
            access_token: access_token.into(),
            refresh_token: None,
            expires_at_ms: None,
            scope: None,
        }
    }

    /// Sets the refresh token.
    pub fn with_refresh_token(mut self, refresh_token: impl Into<String>) -> Self {
        self.refresh_token = Some(refresh_token.into());
        self
    }

    /// Sets the expiry, as Unix epoch milliseconds.
    pub fn with_expires_at_ms(mut self, expires_at_ms: i64) -> Self {
        self.expires_at_ms = Some(expires_at_ms);
        self
    }

    /// Sets the scope string.
    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.scope = Some(scope.into());
        self
    }

    /// True if this token has a known expiry that has already passed, as of
    /// `now_ms` (Unix epoch milliseconds). A token with no known expiry is
    /// never considered expired by this check alone — some issuing
    /// mechanisms (or callers) only learn expiry happened from a 401 on the
    /// actual API call.
    pub fn is_expired(&self, now_ms: i64) -> bool {
        self.expires_at_ms
            .is_some_and(|expires_at| now_ms >= expires_at)
    }

    /// True if this token has no refresh token, meaning a full
    /// re-authentication (not just a refresh) is required once it expires —
    /// the state `nest-auth-oauth-client` is expected to surface distinctly
    /// rather than folding into a generic auth error (see the plan doc's
    /// Schwab-specific notes: a 7-day refresh-token lifetime makes this a
    /// real, recurring case, not an edge case).
    pub fn needs_reauth_on_expiry(&self) -> bool {
        self.refresh_token.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_has_no_refresh_token_expiry_or_scope() {
        let token = Token::new("access-value");

        assert_eq!(token.access_token, "access-value");
        assert_eq!(token.refresh_token, None);
        assert_eq!(token.expires_at_ms, None);
        assert_eq!(token.scope, None);
    }

    #[test]
    fn builder_methods_set_the_expected_fields() {
        let token = Token::new("access-value")
            .with_refresh_token("refresh-value")
            .with_expires_at_ms(1_700_000_000_000)
            .with_scope("read write");

        assert_eq!(token.refresh_token, Some("refresh-value".to_string()));
        assert_eq!(token.expires_at_ms, Some(1_700_000_000_000));
        assert_eq!(token.scope, Some("read write".to_string()));
    }

    #[test]
    fn is_expired_is_false_with_no_known_expiry() {
        let token = Token::new("access-value");

        assert!(!token.is_expired(1_700_000_000_000));
    }

    #[test]
    fn is_expired_compares_against_now_ms() {
        let token = Token::new("access-value").with_expires_at_ms(1_700_000_000_000);

        assert!(!token.is_expired(1_699_999_999_999));
        assert!(token.is_expired(1_700_000_000_000));
        assert!(token.is_expired(1_700_000_000_001));
    }

    #[test]
    fn needs_reauth_on_expiry_reflects_whether_a_refresh_token_is_present() {
        let without_refresh = Token::new("access-value");
        let with_refresh = Token::new("access-value").with_refresh_token("refresh-value");

        assert!(without_refresh.needs_reauth_on_expiry());
        assert!(!with_refresh.needs_reauth_on_expiry());
    }

    #[test]
    fn token_round_trips_through_json() {
        let token = Token::new("access-value")
            .with_refresh_token("refresh-value")
            .with_expires_at_ms(1_700_000_000_000)
            .with_scope("read write");

        let json = serde_json::to_string(&token).expect("serialize");
        let decoded: Token = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(decoded, token);
    }
}
