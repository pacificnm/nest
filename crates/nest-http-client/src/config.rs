//! HTTP client configuration.

use std::sync::Arc;

use nest_http::{AuthStrategy, FixedRetryPolicy, HeaderMap, TimeoutConfig};

/// Configuration for [`crate::HttpClientService`].
#[derive(Clone)]
pub struct HttpClientConfig {
    /// Optional user agent string.
    pub user_agent: Option<String>,
    /// Default connect and request timeouts.
    pub default_timeout: TimeoutConfig,
    /// Default headers applied to every request.
    pub default_headers: HeaderMap,
    /// Optional retry policy.
    pub retry: Option<FixedRetryPolicy>,
    /// Optional authentication strategy.
    pub auth: Option<Arc<dyn AuthStrategy>>,
}

impl Default for HttpClientConfig {
    fn default() -> Self {
        Self {
            user_agent: Some("nest-http-client/0.1".to_string()),
            default_timeout: TimeoutConfig::default(),
            default_headers: HeaderMap::new(),
            retry: None,
            auth: None,
        }
    }
}

impl HttpClientConfig {
    /// Sets the user agent.
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Sets default timeouts.
    pub fn with_timeout(mut self, timeout: TimeoutConfig) -> Self {
        self.default_timeout = timeout;
        self
    }

    /// Sets a default header.
    pub fn with_default_header(mut self, name: impl AsRef<str>, value: impl Into<String>) -> Self {
        self.default_headers.insert(name, value);
        self
    }

    /// Sets the retry policy.
    pub fn with_retry(mut self, retry: FixedRetryPolicy) -> Self {
        self.retry = Some(retry);
        self
    }

    /// Sets the authentication strategy.
    pub fn with_auth(mut self, auth: Arc<dyn AuthStrategy>) -> Self {
        self.auth = Some(auth);
        self
    }
}
