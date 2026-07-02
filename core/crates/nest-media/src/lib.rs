//! Media domain models and provider contracts for the Nest framework.
//!
//! nest-media defines reusable media types — movies, tracks, artwork, metadata
//! search results, and provider traits. It does not perform HTTP, database I/O,
//! filesystem scanning, FFmpeg inspection, or TMDB calls.
//!
//! ## Example
//!
//! ```
//! use nest_media::{MediaId, Movie, MediaItem};
//!
//! let movie = Movie::new(MediaId::new("movie-1"), "Alien");
//! let item = MediaItem::from_movie(&movie);
//! assert_eq!(item.title, "Alien");
//! ```

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

mod artwork;
pub mod codes;
mod error;
mod external;
mod id;
mod inspection;
mod item;
mod kind;
mod metadata;
mod movie;
mod provider;
mod tracks;

pub mod prelude;

pub use artwork::{Artwork, ArtworkKind, ArtworkSource};
pub use error::{MediaError, MediaErrorKind, MediaResult};
pub use external::ExternalIds;
pub use id::{ExternalMediaId, MediaId};
pub use inspection::{MediaInput, MediaInspection};
pub use item::MediaItem;
pub use kind::MediaKind;
pub use metadata::{MovieMetadata, MovieSearchQuery, MovieSearchResult};
pub use movie::{Movie, PersonCredit};
pub use tracks::{AudioTrack, HdrFormat, MediaTracks, SubtitleTrack, VideoTrack};

#[cfg(feature = "async")]
pub use provider::{MediaInspector, MediaLibraryRepository, MetadataProvider};

pub use nest_error::{NestError, NestResult};

impl From<MediaError> for NestError {
    fn from(error: MediaError) -> Self {
        NestError::data(error.message())
            .with_code(error.nest_code())
            .with_module("nest-media")
            .with_source(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nest_error::NestErrorKind;

    #[test]
    fn movie_serde_round_trip() {
        let movie = Movie::new(MediaId::new("movie-1"), "Alien");
        let json = serde_json::to_string(&movie).unwrap();
        let decoded: Movie = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.title, "Alien");
    }

    #[test]
    fn media_item_from_movie() {
        let movie = Movie::new(MediaId::new("movie-1"), "Alien");
        let item = MediaItem::from_movie(&movie);
        assert_eq!(item.kind, MediaKind::Movie);
        assert_eq!(item.title, "Alien");
    }

    #[test]
    fn media_error_converts_to_nest_error() {
        let error = MediaError::not_found("movie missing");
        let nest_error: NestError = error.into();
        assert_eq!(nest_error.kind(), NestErrorKind::Data);
        assert_eq!(nest_error.code(), Some(codes::NEST_MEDIA_NOT_FOUND));
    }

    #[cfg(feature = "async")]
    mod async_traits {
        use super::*;
        use crate::metadata::MovieSearchQuery;
        use crate::provider::{MediaInspector, MediaLibraryRepository, MetadataProvider};

        struct MockProvider;

        #[async_trait::async_trait]
        impl MetadataProvider for MockProvider {
            async fn search_movie(
                &self,
                _query: MovieSearchQuery,
            ) -> MediaResult<Vec<MovieSearchResult>> {
                Ok(Vec::new())
            }

            async fn get_movie(&self, _id: ExternalMediaId) -> MediaResult<MovieMetadata> {
                Err(MediaError::provider("not implemented"))
            }
        }

        struct MockInspector;

        #[async_trait::async_trait]
        impl MediaInspector for MockInspector {
            async fn inspect(&self, _input: MediaInput) -> MediaResult<MediaInspection> {
                Ok(MediaInspection::new(MediaTracks::new()))
            }
        }

        struct MockRepository;

        #[async_trait::async_trait]
        impl MediaLibraryRepository for MockRepository {
            async fn save_movie(&self, _movie: Movie) -> MediaResult<()> {
                Ok(())
            }

            async fn get_movie(&self, _id: MediaId) -> MediaResult<Option<Movie>> {
                Ok(None)
            }

            async fn list_movies(&self) -> MediaResult<Vec<Movie>> {
                Ok(Vec::new())
            }
        }

        #[tokio::test]
        async fn provider_traits_compile() {
            let _provider = MockProvider;
            let _inspector = MockInspector;
            let _repository = MockRepository;
        }
    }
}
