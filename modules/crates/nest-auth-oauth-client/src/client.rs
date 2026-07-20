//! OAuth2 authorization-code + PKCE client, built on the `oauth2` crate.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use nest_auth::Token;
use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, RefreshToken, Scope, TokenResponse, TokenUrl,
};
use url::Url;

use crate::callback::wait_for_callback;
use crate::config::OAuthClientConfig;
use crate::error::{OAuthError, OAuthResult};

/// A pending authorization request: the URL to send the user to, and the
/// CSRF/PKCE state that must survive until the redirect callback arrives.
///
/// Show or open [`AuthorizationRequest::url`] however the caller's UI does
/// that (this crate is UI-agnostic), then pass this whole value to
/// [`OAuthClient::complete_login`].
pub struct AuthorizationRequest {
    /// The URL to send the user to in a browser.
    pub url: Url,
    csrf_token: CsrfToken,
    pkce_verifier: PkceCodeVerifier,
}

/// An OAuth2 authorization-code + PKCE client for a single provider.
pub struct OAuthClient {
    inner: BasicClient,
    scopes: Vec<String>,
    redirect_port: u16,
    use_https_callback: bool,
}

impl OAuthClient {
    /// Builds a client from resolved configuration.
    pub fn new(config: &OAuthClientConfig) -> OAuthResult<Self> {
        let auth_url = AuthUrl::new(config.auth_url.clone()).map_err(|err| {
            OAuthError::config(format!("invalid authorization url: {err}")).with_source(err)
        })?;
        let token_url = TokenUrl::new(config.token_url.clone()).map_err(|err| {
            OAuthError::config(format!("invalid token url: {err}")).with_source(err)
        })?;
        let redirect_url = RedirectUrl::new(config.redirect_uri()).map_err(|err| {
            OAuthError::config(format!("invalid redirect url: {err}")).with_source(err)
        })?;

        let inner = BasicClient::new(
            ClientId::new(config.client_id.clone()),
            config.client_secret.clone().map(ClientSecret::new),
            auth_url,
            Some(token_url),
        )
        .set_redirect_uri(redirect_url);

        Ok(Self {
            inner,
            scopes: config.scopes.clone(),
            redirect_port: config.redirect_port,
            use_https_callback: config.use_https_callback,
        })
    }

    /// Builds the authorization URL and the state needed to complete the
    /// flow. Does not open a browser or listen for the callback — that's
    /// [`OAuthClient::complete_login`]'s job, called after the caller has
    /// shown [`AuthorizationRequest::url`] to the user.
    pub fn authorization_request(&self) -> AuthorizationRequest {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let mut request = self
            .inner
            .authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(pkce_challenge);
        for scope in &self.scopes {
            request = request.add_scope(Scope::new(scope.clone()));
        }
        let (url, csrf_token) = request.url();

        AuthorizationRequest {
            url,
            csrf_token,
            pkce_verifier,
        }
    }

    /// Waits for the provider's redirect on the local loopback listener,
    /// verifies the CSRF state, and exchanges the resulting code for a
    /// [`Token`]. Blocks (async) until the callback arrives or `timeout`
    /// elapses.
    pub async fn complete_login(
        &self,
        request: AuthorizationRequest,
        timeout: Duration,
    ) -> OAuthResult<Token> {
        let callback =
            wait_for_callback(self.redirect_port, timeout, self.use_https_callback).await?;

        if callback.state != *request.csrf_token.secret() {
            return Err(OAuthError::state_mismatch(
                "redirect callback state did not match the authorization request's CSRF token",
            ));
        }

        self.exchange_code(callback.code, request.pkce_verifier)
            .await
    }

    /// Exchanges an authorization code for a token directly, without
    /// running the loopback listener — for callers that receive the code
    /// through some other channel (e.g. a custom URI scheme handler).
    pub async fn exchange_code(
        &self,
        code: impl Into<String>,
        pkce_verifier: PkceCodeVerifier,
    ) -> OAuthResult<Token> {
        let response = self
            .inner
            .exchange_code(AuthorizationCode::new(code.into()))
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client)
            .await
            .map_err(|err| {
                OAuthError::request(format!("token exchange failed: {err}")).with_source(err)
            })?;

        Ok(token_from_response(&response))
    }

    /// Exchanges a refresh token for a new [`Token`].
    pub async fn refresh(&self, refresh_token: &str) -> OAuthResult<Token> {
        let response = self
            .inner
            .exchange_refresh_token(&RefreshToken::new(refresh_token.to_string()))
            .request_async(async_http_client)
            .await
            .map_err(|err| {
                OAuthError::request(format!("token refresh failed: {err}")).with_source(err)
            })?;

        Ok(token_from_response(&response))
    }
}

fn token_from_response(response: &oauth2::basic::BasicTokenResponse) -> Token {
    let mut token = Token::new(response.access_token().secret().clone());

    if let Some(refresh_token) = response.refresh_token() {
        token = token.with_refresh_token(refresh_token.secret().clone());
    }
    if let Some(expires_in) = response.expires_in() {
        token = token.with_expires_at_ms(now_ms() + expires_in.as_millis() as i64);
    }
    if let Some(scopes) = response.scopes() {
        let scope = scopes
            .iter()
            .map(|scope| scope.as_ref())
            .collect::<Vec<&str>>()
            .join(" ");
        if !scope.is_empty() {
            token = token.with_scope(scope);
        }
    }

    token
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_millis() as i64
}

#[cfg(test)]
mod tests {
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;
    use crate::config::OAuthClientConfig;
    use crate::error::OAuthErrorKind;

    fn config_for(server: &MockServer) -> OAuthClientConfig {
        OAuthClientConfig::new(
            "test-client-id",
            format!("{}/authorize", server.uri()),
            format!("{}/token", server.uri()),
        )
        .with_scopes(["read", "write"])
    }

    #[tokio::test]
    async fn exchange_code_maps_a_successful_token_response() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                r#"{"access_token":"access-value","token_type":"Bearer","expires_in":3600,"refresh_token":"refresh-value","scope":"read write"}"#,
                "application/json",
            ))
            .mount(&server)
            .await;

        let client = OAuthClient::new(&config_for(&server)).unwrap();
        let (_, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let before = now_ms();

        let token = client
            .exchange_code("auth-code-value", pkce_verifier)
            .await
            .expect("exchange_code");

        assert_eq!(token.access_token, "access-value");
        assert_eq!(token.refresh_token, Some("refresh-value".to_string()));
        assert_eq!(token.scope, Some("read write".to_string()));
        let expires_at_ms = token.expires_at_ms.expect("expires_at_ms");
        assert!(expires_at_ms >= before + Duration::from_secs(3600).as_millis() as i64);
    }

    #[tokio::test]
    async fn refresh_maps_a_successful_token_response() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(ResponseTemplate::new(200).set_body_raw(
                r#"{"access_token":"new-access-value","token_type":"Bearer","expires_in":1800}"#,
                "application/json",
            ))
            .mount(&server)
            .await;

        let client = OAuthClient::new(&config_for(&server)).unwrap();

        let token = client.refresh("refresh-value").await.expect("refresh");

        assert_eq!(token.access_token, "new-access-value");
        assert_eq!(token.refresh_token, None);
        assert!(token.expires_at_ms.is_some());
    }

    #[tokio::test]
    async fn exchange_code_surfaces_provider_errors_as_a_request_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .respond_with(
                ResponseTemplate::new(400).set_body_string(r#"{"error":"invalid_grant"}"#),
            )
            .mount(&server)
            .await;

        let client = OAuthClient::new(&config_for(&server)).unwrap();
        let (_, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let error = client
            .exchange_code("bad-code", pkce_verifier)
            .await
            .expect_err("should error");

        assert_eq!(error.kind(), OAuthErrorKind::Request);
    }
}
