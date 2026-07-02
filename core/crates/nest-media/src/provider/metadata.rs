//! Metadata provider contract.

use async_trait::async_trait;

use crate::error::MediaResult;
use crate::id::ExternalMediaId;
use crate::metadata::{MovieMetadata, MovieSearchQuery, MovieSearchResult};

/// Fetches movie metadata from an external provider.
#[async_trait]
pub trait MetadataProvider: Send + Sync {
    /// Searches for movies matching the query.
    async fn search_movie(&self, query: MovieSearchQuery) -> MediaResult<Vec<MovieSearchResult>>;

    /// Fetches full movie metadata for a provider id.
    async fn get_movie(&self, id: ExternalMediaId) -> MediaResult<MovieMetadata>;
}
