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
    /// `http://127.0.0.1:<redirect_port>/callback`.
    pub redirect_port: u16,
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

    /// The redirect URI the loopback listener answers on, and that must be
    /// registered with the provider as this client's redirect URI.
    pub fn redirect_uri(&self) -> String {
        format!("http://127.0.0.1:{}/callback", self.redirect_port)
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
    /// Requested scopes.
    #[serde(default)]
    pub scopes: Vec<String>,
}

#[cfg(feature = "config")]
impl OAuthClientSection {
    fn into_config(self) -> OAuthClientConfig {
        let mut config = OAuthClientConfig::new(self.client_id, self.auth_url, self.token_url)
            .with_redirect_port(self.redirect_port)
            .with_scopes(self.scopes);
        if let Some(client_secret) = self.client_secret {
            config = config.with_client_secret(client_secret);
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
