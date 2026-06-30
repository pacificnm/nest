//! Optional Nest module that registers [`crate::HttpClientService`].

use nest_core::{AppBuilder, Module, ModuleId, NestResult};

use crate::config::HttpClientConfig;
use crate::service::HttpClientService;

/// Module id for [`HttpClientModule`].
pub const HTTP_CLIENT_MODULE_ID: ModuleId = ModuleId("nest-http-client");

/// Registers [`HttpClientService`].
pub struct HttpClientModule {
    config: HttpClientConfig,
}

impl HttpClientModule {
    /// Creates a module with default client configuration.
    pub fn new() -> Self {
        Self {
            config: HttpClientConfig::default(),
        }
    }

    /// Creates a module with custom configuration.
    pub fn with_config(config: HttpClientConfig) -> Self {
        Self { config }
    }

    /// Sets the user agent.
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.config = self.config.with_user_agent(user_agent);
        self
    }

    /// Sets default timeouts.
    pub fn with_timeout(mut self, timeout: nest_http::TimeoutConfig) -> Self {
        self.config = self.config.with_timeout(timeout);
        self
    }

    /// Sets the retry policy.
    pub fn with_retry(mut self, retry: nest_http::FixedRetryPolicy) -> Self {
        self.config = self.config.with_retry(retry);
        self
    }

    /// Sets the authentication strategy.
    pub fn with_auth(mut self, auth: std::sync::Arc<dyn nest_http::AuthStrategy>) -> Self {
        self.config = self.config.with_auth(auth);
        self
    }
}

impl Default for HttpClientModule {
    fn default() -> Self {
        Self::new()
    }
}

impl Module for HttpClientModule {
    fn id(&self) -> ModuleId {
        HTTP_CLIENT_MODULE_ID
    }

    fn configure(&self, app: &mut AppBuilder) -> NestResult<()> {
        let service = HttpClientService::new(self.config.clone())?;
        app.register_service(service)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nest_core::AppBuilder;

    #[test]
    fn module_registers_http_client_service() {
        let built = AppBuilder::new()
            .module(HttpClientModule::default())
            .build()
            .unwrap();
        assert!(built.context.has_service::<HttpClientService>());
    }
}
