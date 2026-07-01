//! TMDB metadata provider adapter for the Nest framework.
//!
//! `nest-tmdb` translates TMDB API responses into [`nest_media`] types and
//! implements [`nest_media::MetadataProvider`]. It does not own media domain
//! models or serve HTTP.
//!
//! # Quick start
//!
//! ```no_run
//! use nest_core::AppBuilder;
//! use nest_http_client::HttpClientModule;
//! use nest_media::{MetadataProvider, MovieSearchQuery};
//! use nest_tmdb::{TmdbConfig, TmdbMetadataProvider, TmdbModule};
//!
//! let built = AppBuilder::new()
//!     .module(HttpClientModule::default())
//!     .module(TmdbModule::with_config(
//!         TmdbConfig::builder().api_key("your-key").build().unwrap(),
//!     ))
//!     .build()
//!     .unwrap();
//!
//! let provider = built.context.service::<TmdbMetadataProvider>().unwrap();
//! # async fn demo(provider: &TmdbMetadataProvider) -> nest_media::MediaResult<()> {
//! let results = provider
//!     .search_movie(MovieSearchQuery::new("Alien").with_year(1979))
//!     .await?;
//! let metadata = provider.get_movie(results[0].external_id.clone()).await?;
//! # Ok(())
//! # }
//! ```

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

mod client;
mod codes;
mod config;
mod dto;
mod error;
mod images;
mod mapper;
mod module;
mod provider;

pub mod prelude;

pub use client::TmdbClient;
pub use config::{
    resolve_api_key, TmdbConfig, TmdbConfigBuilder, DEFAULT_API_KEY_ENV, DEFAULT_BASE_URL,
    DEFAULT_IMAGE_BASE_URL, DEFAULT_LANGUAGE,
};
pub use error::{tmdb_to_media_error, TmdbError, TmdbErrorKind, TmdbResult};
pub use images::{artwork_for_movie_with_base, ImageSize, TmdbImageService};
pub use mapper::{external_id_for_movie, parse_movie_external_id};
pub use module::{TmdbModule, TMDB_MODULE_ID};
pub use provider::TmdbMetadataProvider;

pub use nest_error::{NestError, NestResult};
pub use nest_media::{MetadataProvider, MovieMetadata, MovieSearchQuery, MovieSearchResult};

impl From<TmdbError> for NestError {
    fn from(error: TmdbError) -> Self {
        NestError::network(error.message())
            .with_code(error.nest_code())
            .with_module("nest-tmdb")
            .with_source(error)
    }
}

#[cfg(test)]
mod tests {
    use nest_core::AppBuilder;
    use nest_error::NestErrorKind;
    use nest_http_client::HttpClientModule;
    use nest_media::{ExternalMediaId, MovieSearchQuery};
    use serde_json::json;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;
    use crate::config::TmdbConfig;

    fn test_config(base_uri: &str) -> TmdbConfig {
        TmdbConfig::builder()
            .api_key("test-key")
            .base_url(base_uri)
            .build()
            .unwrap()
    }

    fn test_provider(config: TmdbConfig) -> TmdbMetadataProvider {
        let built = AppBuilder::new()
            .module(HttpClientModule::default())
            .module(TmdbModule::with_config(config))
            .build()
            .unwrap();
        built
            .context
            .service::<TmdbMetadataProvider>()
            .unwrap()
            .clone()
    }

    #[test]
    fn tmdb_error_converts_to_nest_error() {
        let tmdb_error = TmdbError::api("request failed");
        let nest_error: NestError = tmdb_error.into();
        assert_eq!(nest_error.kind(), NestErrorKind::Network);
        assert_eq!(nest_error.code(), Some(codes::NEST_TMDB_API_ERROR));
    }

    #[tokio::test]
    async fn search_movie_returns_mapped_results() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/search/movie"))
            .and(query_param("api_key", "test-key"))
            .and(query_param("query", "Alien"))
            .and(query_param("year", "1979"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "results": [{
                    "id": 348,
                    "title": "Alien",
                    "overview": "In space no one can hear you scream.",
                    "release_date": "1979-05-25"
                }]
            })))
            .mount(&server)
            .await;

        let provider = test_provider(test_config(&server.uri()));
        let results = provider
            .search_movie(MovieSearchQuery::new("Alien").with_year(1979))
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].external_id.as_str(), "tmdb:348");
        assert_eq!(results[0].title, "Alien");
        assert_eq!(results[0].year, Some(1979));
    }

    #[tokio::test]
    async fn get_movie_fetches_details_credits_and_external_ids() {
        let server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/configuration"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "images": {
                    "base_url": "https://image.tmdb.org/t/p/",
                    "poster_sizes": ["w500"],
                    "backdrop_sizes": ["w1280"]
                }
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/movie/348"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": 348,
                "title": "Alien",
                "original_title": "Alien",
                "overview": "Summary",
                "release_date": "1979-05-25",
                "runtime": 117,
                "poster_path": "/poster.jpg",
                "backdrop_path": "/backdrop.jpg",
                "genres": [{ "id": 27, "name": "Horror" }]
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/movie/348/credits"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "cast": [{
                    "name": "Sigourney Weaver",
                    "character": "Ripley",
                    "order": 0
                }],
                "crew": [{
                    "name": "Ridley Scott",
                    "job": "Director"
                }]
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/movie/348/external_ids"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "imdb_id": "tt0078748",
                "tmdb_id": 348
            })))
            .mount(&server)
            .await;

        let provider = test_provider(test_config(&server.uri()));
        let metadata = provider
            .get_movie(ExternalMediaId::new("tmdb:348"))
            .await
            .unwrap();

        assert_eq!(metadata.title, "Alien");
        assert_eq!(metadata.year, Some(1979));
        assert_eq!(metadata.runtime_seconds, Some(117 * 60));
        assert_eq!(metadata.genres, vec!["Horror"]);
        assert_eq!(metadata.cast[0].name, "Sigourney Weaver");
        assert_eq!(metadata.crew[0].role, "Director");
        assert_eq!(metadata.external_ids.imdb_id.as_deref(), Some("tt0078748"));
    }
}
