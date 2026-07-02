//! nest-media MetadataProvider implementation.

use async_trait::async_trait;
use nest_media::{
    ExternalMediaId, MediaResult, MetadataProvider, MovieMetadata, MovieSearchQuery,
    MovieSearchResult,
};
use tracing::instrument;

use crate::client::TmdbClient;
use crate::error::tmdb_to_media_error;
use crate::mapper::{
    artwork_paths, map_movie_metadata, map_search_results, parse_movie_external_id,
};

/// Full movie fetch including TMDB artwork path tokens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MovieFetchResult {
    /// Provider-normalized movie metadata.
    pub metadata: MovieMetadata,
    /// TMDB poster path token (e.g. `/abc.jpg`).
    pub poster_path: Option<String>,
    /// TMDB backdrop path token.
    pub backdrop_path: Option<String>,
}

/// TMDB-backed metadata provider for nest-media.
#[derive(Clone)]
pub struct TmdbMetadataProvider {
    client: TmdbClient,
}

/// TMDB person details for app-layer caching.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonDetails {
    /// TMDB person id.
    pub id: u32,
    /// Display name.
    pub name: String,
    /// Biography text.
    pub biography: Option<String>,
    /// Birthday ISO date.
    pub birthday: Option<String>,
    /// Deathday ISO date.
    pub deathday: Option<String>,
    /// Birth place label.
    pub place_of_birth: Option<String>,
    /// TMDB profile path token.
    pub profile_path: Option<String>,
    /// Primary department.
    pub known_for_department: Option<String>,
    /// TMDB gender code.
    pub gender: Option<i32>,
    /// Alternate names.
    pub also_known_as: Vec<String>,
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

    /// Fetches movie metadata and artwork path tokens in one TMDB round-trip set.
    #[instrument(skip(self, id), fields(external_id = %id))]
    pub async fn fetch_movie(&self, id: ExternalMediaId) -> MediaResult<MovieFetchResult> {
        let movie_id = parse_movie_external_id(&id)?;
        self.client
            .ensure_configuration()
            .await
            .map_err(tmdb_to_media_error)?;

        let movie = self
            .client
            .movie_details(movie_id)
            .await
            .map_err(tmdb_to_media_error)?;
        let (poster_path, backdrop_path) = artwork_paths(&movie);
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

        Ok(MovieFetchResult {
            metadata: map_movie_metadata(movie, credits, external_ids),
            poster_path,
            backdrop_path,
        })
    }

    /// Fetches TMDB person details.
    pub async fn fetch_person(&self, person_id: u32) -> MediaResult<PersonDetails> {
        self.client
            .ensure_configuration()
            .await
            .map_err(tmdb_to_media_error)?;
        let person = self
            .client
            .person_details(person_id)
            .await
            .map_err(tmdb_to_media_error)?;
        Ok(PersonDetails {
            id: person.id,
            name: person.name,
            biography: person.biography.filter(|value| !value.is_empty()),
            birthday: person.birthday.filter(|value| !value.is_empty()),
            deathday: person.deathday.filter(|value| !value.is_empty()),
            place_of_birth: person.place_of_birth.filter(|value| !value.is_empty()),
            profile_path: person.profile_path.filter(|value| !value.is_empty()),
            known_for_department: person
                .known_for_department
                .filter(|value| !value.is_empty()),
            gender: person.gender,
            also_known_as: person
                .also_known_as
                .unwrap_or_default()
                .into_iter()
                .filter(|value| !value.is_empty())
                .collect(),
        })
    }
}

#[async_trait]
impl MetadataProvider for TmdbMetadataProvider {
    #[instrument(skip(self, query), fields(query = %query.query))]
    async fn search_movie(&self, query: MovieSearchQuery) -> MediaResult<Vec<MovieSearchResult>> {
        let response = self
            .client
            .search_movie(&query)
            .await
            .map_err(tmdb_to_media_error)?;
        Ok(map_search_results(response.results))
    }

    #[instrument(skip(self, id), fields(external_id = %id))]
    async fn get_movie(&self, id: ExternalMediaId) -> MediaResult<MovieMetadata> {
        Ok(self.fetch_movie(id).await?.metadata)
    }
}
