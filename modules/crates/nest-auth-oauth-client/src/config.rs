//! OAuth client configuration.

#![allow(clippy::result_large_err)]

#[cfg(feature = "config")]
use nest_config::ConfigService;
#[cfg(feature = "config")]
use nest_error::NestResult;
#[cfg(feature = "config")]
use serde::Deserialize;

/// Default loopback port the redirect callback listens on.
pub const DEFAULT_REDIRECT_PORT: u16 = 51_763;

/// Resolved OAuth client configuration.
#[derive(Debug, Clone)]
pub struct OAuthClientConfig {
    /// OAuth2 client id.
    pub client_id: String,
    /// OAuth2 client secret, if the provider requires one (confidential
    /// clients only — most desktop-app flows are public clients and omit
    /// this, relying on PKCE instead).
    pub client_secret: Option<String>,
    /// Authorization endpoint URL.
    pub auth_url: String,
    /// Token endpoint URL.
    pub token_url: String,
    /// Port the local loopback redirect listener binds to. The redirect
    /// URI registered with the provider must be
    /// `http://127.0.0.1:<redirect_port>/callback` (or `https://` — see
    /// [`Self::use_https_callback`] — and see [`Self::redirect_uri_override`]
    /// if the provider requires a different path or no path at all).
    pub redirect_port: u16,
    /// Terminates the loopback listener with a locally generated
    /// self-signed certificate instead of plain HTTP. Some providers (e.g.
    /// Schwab) require an `https://` redirect URI even for a `127.0.0.1`
    /// loopback — the browser will show a self-signed-certificate warning,
    /// which is expected and safe to proceed through.
    pub use_https_callback: bool,
    /// Overrides the derived `redirect_uri()` entirely. Use this when the
    /// provider's registered redirect URI doesn't match this crate's
    /// default `<scheme>://127.0.0.1:<redirect_port>/callback` shape (for
    /// example, Schwab app registrations commonly use a bare
    /// `https://127.0.0.1:<port>` with no path) — the provider requires an
    /// exact string match, so this must be set to precisely what's
    /// registered.
    pub redirect_uri_override: Option<String>,
    /// Scopes to request.
    pub scopes: Vec<String>,
}

impl OAuthClientConfig {
    /// Creates configuration with no scopes and the default redirect port.
    pub fn new(
        client_id: impl Into<String>,
        auth_url: impl Into<String>,
        token_url: impl Into<String>,
    ) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret: None,
            auth_url: auth_url.into(),
            token_url: token_url.into(),
            redirect_port: DEFAULT_REDIRECT_PORT,
            use_https_callback: false,
            redirect_uri_override: None,
            scopes: Vec::new(),
        }
    }

    /// Sets the client secret.
    pub fn with_client_secret(mut self, client_secret: impl Into<String>) -> Self {
        self.client_secret = Some(client_secret.into());
        self
    }

    /// Sets the loopback redirect port.
    pub fn with_redirect_port(mut self, redirect_port: u16) -> Self {
        self.redirect_port = redirect_port;
        self
    }

    /// Sets the requested scopes.
    pub fn with_scopes(mut self, scopes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.scopes = scopes.into_iter().map(Into::into).collect();
        self
    }

    /// Enables terminating the loopback listener with a self-signed
    /// certificate so the redirect URI is `https://`, as some providers
    /// (e.g. Schwab) require.
    pub fn with_https_callback(mut self, use_https_callback: bool) -> Self {
        self.use_https_callback = use_https_callback;
        self
    }

    /// Overrides the derived redirect URI with an exact string. Required
    /// when the provider's registered redirect URI isn't
    /// `<scheme>://127.0.0.1:<redirect_port>/callback`.
    pub fn with_redirect_uri(mut self, redirect_uri: impl Into<String>) -> Self {
        self.redirect_uri_override = Some(redirect_uri.into());
        self
    }

    /// The redirect URI the loopback listener answers on, and that must be
    /// registered with the provider as this client's redirect URI (exactly,
    /// if [`Self::redirect_uri_override`] isn't set to something else).
    pub fn redirect_uri(&self) -> String {
        if let Some(redirect_uri) = &self.redirect_uri_override {
            return redirect_uri.clone();
        }
        let scheme = if self.use_https_callback {
            "https"
        } else {
            "http"
        };
        format!("{scheme}://127.0.0.1:{}/callback", self.redirect_port)
    }

    /// Loads the `[oauth_client]` section from a config service.
    #[cfg(feature = "config")]
    pub fn from_config_service(service: &ConfigService) -> NestResult<Option<Self>> {
        let Ok(section) = service.section::<OAuthClientSection>("oauth_client") else {
            return Ok(None);
        };
        if !section.enabled {
            return Ok(None);
        }
        Ok(Some(section.into_config()))
    }
}

/// `[oauth_client]` config section.
#[cfg(feature = "config")]
#[derive(Debug, Clone, Deserialize)]
pub struct OAuthClientSection {
    /// Whether the OAuth client is configured/enabled.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// OAuth2 client id.
    pub client_id: String,
    /// OAuth2 client secret, if required.
    pub client_secret: Option<String>,
    /// Authorization endpoint URL.
    pub auth_url: String,
    /// Token endpoint URL.
    pub token_url: String,
    /// Loopback redirect port.
    #[serde(default = "default_redirect_port")]
    pub redirect_port: u16,
    /// Whether to terminate the loopback listener with a self-signed cert.
    #[serde(default)]
    pub use_https_callback: bool,
    /// Overrides the derived redirect URI.
    pub redirect_uri: Option<String>,
    /// Requested scopes.
    #[serde(default)]
    pub scopes: Vec<String>,
}

#[cfg(feature = "config")]
impl OAuthClientSection {
    fn into_config(self) -> OAuthClientConfig {
        let mut config = OAuthClientConfig::new(self.client_id, self.auth_url, self.token_url)
            .with_redirect_port(self.redirect_port)
            .with_https_callback(self.use_https_callback)
            .with_scopes(self.scopes);
        if let Some(client_secret) = self.client_secret {
            config = config.with_client_secret(client_secret);
        }
        if let Some(redirect_uri) = self.redirect_uri {
            config = config.with_redirect_uri(redirect_uri);
        }
        config
    }
}

#[cfg(feature = "config")]
fn default_enabled() -> bool {
    true
}

#[cfg(feature = "config")]
fn default_redirect_port() -> u16 {
    DEFAULT_REDIRECT_PORT
}
