//! Common imports for nest-tmdb consumers.

pub use crate::client::TmdbClient;
pub use crate::config::{TmdbConfig, TmdbConfigBuilder, DEFAULT_API_KEY_ENV, DEFAULT_BASE_URL};
pub use crate::error::{TmdbError, TmdbErrorKind, TmdbResult};
pub use crate::images::{ImageSize, TmdbImageService};
pub use crate::mapper::{external_id_for_movie, parse_movie_external_id};
pub use crate::module::{TmdbModule, TMDB_MODULE_ID};
pub use crate::provider::TmdbMetadataProvider;

pub use nest_error::{NestError, NestResult};
pub use nest_media::{MetadataProvider, MovieMetadata, MovieSearchQuery, MovieSearchResult};
