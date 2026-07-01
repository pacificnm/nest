//! nest-media MetadataProvider implementation.

use async_trait::async_trait;
use nest_media::{
    ExternalMediaId, MediaResult, MetadataProvider, MovieMetadata, MovieSearchQuery,
    MovieSearchResult,
};
use tracing::instrument;

use crate::client::TmdbClient;
use crate::error::tmdb_to_media_error;
use crate::mapper::{map_movie_metadata, map_search_results, parse_movie_external_id};

/// TMDB-backed metadata provider for nest-media.
#[derive(Clone)]
pub struct TmdbMetadataProvider {
    client: TmdbClient,
}

impl TmdbMetadataProvider {
    /// Creates a provider backed by the given client.
    pub fn new(client: TmdbClient) -> Self {
        Self { client }
    }

    /// Returns the underlying TMDB client.
    pub fn client(&self) -> &TmdbClient {
        &self.client
    }
}

#[async_trait]
impl MetadataProvider for TmdbMetadataProvider {
    #[instrument(skip(self, query), fields(query = %query.query))]
    async fn search_movie(
        &self,
        query: MovieSearchQuery,
    ) -> MediaResult<Vec<MovieSearchResult>> {
        let response = self
            .client
            .search_movie(&query)
            .await
            .map_err(tmdb_to_media_error)?;
        Ok(map_search_results(response.results))
    }

    #[instrument(skip(self, id), fields(external_id = %id))]
    async fn get_movie(&self, id: ExternalMediaId) -> MediaResult<MovieMetadata> {
        let movie_id = parse_movie_external_id(&id)?;
        self.client.ensure_configuration().await.map_err(tmdb_to_media_error)?;

        let movie = self
            .client
            .movie_details(movie_id)
            .await
            .map_err(tmdb_to_media_error)?;
        let credits = self
            .client
            .movie_credits(movie_id)
            .await
            .map_err(tmdb_to_media_error)?;
        let external_ids = self
            .client
            .movie_external_ids(movie_id)
            .await
            .map_err(tmdb_to_media_error)?;

        Ok(map_movie_metadata(movie, credits, external_ids))
    }
}
