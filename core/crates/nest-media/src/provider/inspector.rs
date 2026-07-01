//! Media inspection contract.

use async_trait::async_trait;

use crate::error::MediaResult;
use crate::inspection::{MediaInput, MediaInspection};

/// Inspects local media files and returns technical metadata.
#[async_trait]
pub trait MediaInspector: Send + Sync {
    /// Inspects one media input.
    async fn inspect(&self, input: MediaInput) -> MediaResult<MediaInspection>;
}
