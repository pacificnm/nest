//! Schwab client configuration.
//!
//! Schwab-specific endpoints/defaults live here, not in `nest-auth-oauth-client`
//! itself — see `docs/nest-auth/plan.md`'s "Schwab-specific behavior" section.

#![allow(clippy::result_large_err)]

use nest_auth_oauth_client::OAuthClientConfig;
#[cfg(feature = "config")]
use nest_config::ConfigService;
#[cfg(feature = "config")]
use nest_error::NestResult;
#[cfg(feature = "config")]
use serde::Deserialize;

/// Schwab's OAuth2 authorization endpoint.
pub const SCHWAB_AUTH_URL: &str = "https://api.schwabapi.com/v1/oauth/authorize";
/// Schwab's OAuth2 token endpoint.
pub const SCHWAB_TOKEN_URL: &str = "https://api.schwabapi.com/v1/oauth/token";
/// Base URL for the Accounts and Trading Production API.
pub const SCHWAB_TRADER_BASE_URL: &str = "https://api.schwabapi.com/trader/v1";
/// Base URL for the Market Data Production API.
pub const SCHWAB_MARKET_DATA_BASE_URL: &str = "https://api.schwabapi.com/marketdata/v1";
/// Schwab's documented OAuth scope.
pub const SCHWAB_OAUTH_SCOPE: &str = "api";
/// Default loopback port for the HTTPS redirect callback.
pub const DEFAULT_REDIRECT_PORT: u16 = 8182;

/// Resolved Schwab client configuration.
#[derive(Debug, Clone)]
pub struct SchwabConfig {
    /// Schwab app key (OAuth2 client id).
    pub app_key: String,
    /// Schwab app secret (OAuth2 client secret — Schwab is always a
    /// confidential client).
    pub app_secret: String,
    /// Port the local HTTPS loopback listener binds to. Must match what's
    /// registered as this app's callback URL in the Schwab developer
    /// console (`https://127.0.0.1:<redirect_port>`, no path, unless
    /// [`Self::redirect_uri_override`] is set to something else).
    pub redirect_port: u16,
    /// Overrides the derived `https://127.0.0.1:<redirect_port>` redirect
    /// URI with an exact string — set this if the app's registered
    /// callback URL isn't that bare form.
    pub redirect_uri_override: Option<String>,
    /// Base URL for the Accounts and Trading API. Overridable for testing
    /// against a mock server.
    pub trader_base_url: String,
    /// Base URL for the Market Data API. Overridable for testing against a
    /// mock server.
    pub market_data_base_url: String,
}

impl SchwabConfig {
    /// Creates configuration with Schwab's real endpoints and the default
    /// redirect port.
    pub fn new(app_key: impl Into<String>, app_secret: impl Into<String>) -> Self {
        Self {
            app_key: app_key.into(),
            app_secret: app_secret.into(),
            redirect_port: DEFAULT_REDIRECT_PORT,
            redirect_uri_override: None,
            trader_base_url: SCHWAB_TRADER_BASE_URL.to_string(),
            market_data_base_url: SCHWAB_MARKET_DATA_BASE_URL.to_string(),
        }
    }

    /// Sets the loopback redirect port.
    pub fn with_redirect_port(mut self, redirect_port: u16) -> Self {
        self.redirect_port = redirect_port;
        self
    }

    /// Overrides the derived redirect URI with an exact string.
    pub fn with_redirect_uri(mut self, redirect_uri: impl Into<String>) -> Self {
        self.redirect_uri_override = Some(redirect_uri.into());
        self
    }

    /// Overrides the Trader API base URL (for pointing at a mock server in tests).
    pub fn with_trader_base_url(mut self, trader_base_url: impl Into<String>) -> Self {
        self.trader_base_url = trader_base_url.into();
        self
    }

    /// Overrides the Market Data API base URL (for pointing at a mock
    /// server in tests).
    pub fn with_market_data_base_url(mut self, market_data_base_url: impl Into<String>) -> Self {
        self.market_data_base_url = market_data_base_url.into();
        self
    }

    /// The redirect URI that must be registered as this app's callback URL
    /// in the Schwab developer console.
    pub fn redirect_uri(&self) -> String {
        self.redirect_uri_override
            .clone()
            .unwrap_or_else(|| format!("https://127.0.0.1:{}", self.redirect_port))
    }

    /// Builds the generic [`OAuthClientConfig`] this crate's `OAuthClient`
    /// is constructed from, with Schwab's endpoints, scope, and HTTPS
    /// callback requirement pre-filled.
    pub fn to_oauth_client_config(&self) -> OAuthClientConfig {
        OAuthClientConfig::new(self.app_key.clone(), SCHWAB_AUTH_URL, SCHWAB_TOKEN_URL)
            .with_client_secret(self.app_secret.clone())
            .with_redirect_port(self.redirect_port)
            .with_redirect_uri(self.redirect_uri())
            .with_https_callback(true)
            .with_scopes([SCHWAB_OAUTH_SCOPE])
    }

    /// Loads the `[schwab]` section from a config service.
    #[cfg(feature = "config")]
    pub fn from_config_service(service: &ConfigService) -> NestResult<Option<Self>> {
        let Ok(section) = service.section::<SchwabSection>("schwab") else {
            return Ok(None);
        };
        if !section.enabled {
            return Ok(None);
        }
        Ok(Some(section.into_config()))
    }
}

/// `[schwab]` config section.
#[cfg(feature = "config")]
#[derive(Debug, Clone, Deserialize)]
pub struct SchwabSection {
    /// Whether the Schwab client is configured/enabled.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Schwab app key.
    pub app_key: String,
    /// Schwab app secret.
    pub app_secret: String,
    /// Loopback redirect port.
    #[serde(default = "default_redirect_port")]
    pub redirect_port: u16,
    /// Overrides the derived redirect URI.
    pub redirect_uri: Option<String>,
}

#[cfg(feature = "config")]
impl SchwabSection {
    fn into_config(self) -> SchwabConfig {
        let mut config =
            SchwabConfig::new(self.app_key, self.app_secret).with_redirect_port(self.redirect_port);
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
