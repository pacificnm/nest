//! Registered AI inference service.

use std::sync::Arc;

use crate::error::AiResult;
use crate::provider::AiProvider;
use crate::stream::CompletionStream;
use crate::types::{CompletionRequest, CompletionResponse};

/// Application-scoped AI provider registered by provider modules.
#[derive(Clone)]
pub struct AiService {
    inner: Arc<dyn AiProvider>,
}

impl AiService {
    /// Wraps a provider for dependency injection.
    pub fn new(provider: Arc<dyn AiProvider>) -> Self {
        Self { inner: provider }
    }

    /// Returns the stable provider id.
    pub fn provider_id(&self) -> &'static str {
        self.inner.provider_id()
    }

    /// Runs a completion request against the configured provider.
    pub async fn complete(&self, request: CompletionRequest) -> AiResult<CompletionResponse> {
        self.inner.complete(request).await
    }

    /// Streams a completion when the provider supports it.
    pub async fn stream_complete(
        &self,
        request: CompletionRequest,
    ) -> AiResult<CompletionStream> {
        self.inner.stream_complete(request).await
    }
}
