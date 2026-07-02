//! Low-level TMDB HTTP client.

use std::sync::Arc;

use nest_error::NestResult;
use nest_http_client::HttpClientService;
use nest_media::MovieSearchQuery;
use tokio::sync::RwLock;
use tracing::{debug, instrument};

use crate::config::{TmdbConfig, DEFAULT_IMAGE_BASE_URL};
use crate::dto::configuration::ConfigurationResponse;
use crate::dto::person::PersonDetailsResponse;
use crate::dto::credits::MovieCreditsResponse;
use crate::dto::external_ids::MovieExternalIdsResponse;
use crate::dto::movie::MovieDetailsResponse;
use crate::dto::search::MovieSearchResponse;
use crate::error::{TmdbError, TmdbResult};

/// TMDB v3 API client.
#[derive(Clone)]
pub struct TmdbClient {
    http: HttpClientService,
    config: TmdbConfig,
    image_base_url: Arc<RwLock<String>>,
}

impl TmdbClient {
    /// Creates a TMDB client.
    pub fn new(http: HttpClientService, config: TmdbConfig) -> NestResult<Self> {
        let image_base_url = Arc::new(RwLock::new(config.image_base_url.clone()));
        Ok(Self {
            http,
            config,
            image_base_url,
        })
    }

    /// Returns the resolved configuration.
    pub fn config(&self) -> &TmdbConfig {
        &self.config
    }

    /// Returns the shared image base URL handle used by [`crate::TmdbImageService`].
    pub fn image_base_url(&self) -> Arc<RwLock<String>> {
        self.image_base_url.clone()
    }

    /// Searches movies via `GET /search/movie`.
    #[instrument(skip(self, query), fields(query = %query.query))]
    pub(crate) async fn search_movie(
        &self,
        query: &MovieSearchQuery,
    ) -> TmdbResult<MovieSearchResponse> {
        let mut params = vec![
            ("api_key", self.config.api_key.as_str()),
            ("language", self.config.language.as_str()),
            ("query", query.query.as_str()),
        ];
        let year_param = query.year.map(|year| year.to_string());
        if let Some(year) = &year_param {
            params.push(("year", year.as_str()));
        }
        if let Some(region) = &self.config.region {
            params.push(("region", region.as_str()));
        }

        let url = format!(
            "{}/search/movie?{}",
            self.config.base_url,
            encode_query(&params)
        );

        debug!("tmdb search movie");
        self.get_json(&url).await
    }

    /// Fetches movie details via `GET /movie/{id}`.
    #[instrument(skip(self))]
    pub(crate) async fn movie_details(&self, movie_id: u32) -> TmdbResult<MovieDetailsResponse> {
        let url = self.movie_url(movie_id, &[]);
        self.get_json(&url).await
    }

    /// Fetches movie credits via `GET /movie/{id}/credits`.
    pub(crate) async fn movie_credits(&self, movie_id: u32) -> TmdbResult<MovieCreditsResponse> {
        let url = self.movie_url(movie_id, &["credits"]);
        self.get_json(&url).await
    }

    /// Fetches movie external ids via `GET /movie/{id}/external_ids`.
    pub(crate) async fn movie_external_ids(
        &self,
        movie_id: u32,
    ) -> TmdbResult<MovieExternalIdsResponse> {
        let url = self.movie_url(movie_id, &["external_ids"]);
        self.get_json(&url).await
    }

    /// Fetches person details via `GET /person/{id}`.
    pub(crate) async fn person_details(
        &self,
        person_id: u32,
    ) -> TmdbResult<PersonDetailsResponse> {
        let url = format!(
            "{}/person/{person_id}?{}",
            self.config.base_url,
            encode_query(&[
                ("api_key", self.config.api_key.as_str()),
                ("language", self.config.language.as_str()),
            ])
        );
        self.get_json(&url).await
    }

    /// Loads TMDB configuration and updates the shared image base URL.
    pub async fn load_configuration(&self) -> TmdbResult<()> {
        let url = format!(
            "{}/configuration?{}",
            self.config.base_url,
            encode_query(&[
                ("api_key", self.config.api_key.as_str()),
                ("language", self.config.language.as_str()),
            ])
        );
        let response: ConfigurationResponse = self.get_json(&url).await?;
        let mut base_url = response.images.base_url;
        if !base_url.ends_with('/') {
            base_url.push('/');
        }
        *self.image_base_url.write().await = base_url;
        Ok(())
    }

    /// Ensures image configuration has been loaded at least once.
    pub async fn ensure_configuration(&self) -> TmdbResult<()> {
        let current = self.image_base_url.read().await;
        if current.as_str() == DEFAULT_IMAGE_BASE_URL || current.is_empty() {
            drop(current);
            self.load_configuration().await?;
        }
        Ok(())
    }

    async fn get_json<T>(&self, url: &str) -> TmdbResult<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.http.get_json(url).await.map_err(TmdbError::from)
    }

    fn movie_url(&self, movie_id: u32, suffix: &[&str]) -> String {
        let mut path = format!("{}/movie/{movie_id}", self.config.base_url);
        for segment in suffix {
            path.push('/');
            path.push_str(segment);
        }
        let query = encode_query(&[
            ("api_key", self.config.api_key.as_str()),
            ("language", self.config.language.as_str()),
        ]);
        format!("{path}?{query}")
    }
}

fn encode_query(params: &[(&str, &str)]) -> String {
    params
        .iter()
        .map(|(key, value)| format!("{key}={}", percent_encode(value)))
        .collect::<Vec<_>>()
        .join("&")
}

fn percent_encode(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (byte as char).to_string()
            }
            b' ' => "+".to_string(),
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_encodes_spaces() {
        assert_eq!(percent_encode("Alien 1979"), "Alien+1979");
    }
}
