//! Media library repository contract.

use async_trait::async_trait;

use crate::error::MediaResult;
use crate::id::MediaId;
use crate::movie::Movie;

/// Persists and retrieves movies in a media library.
#[async_trait]
pub trait MediaLibraryRepository: Send + Sync {
    /// Saves or updates a movie.
    async fn save_movie(&self, movie: Movie) -> MediaResult<()>;

    /// Returns one movie by Nest id.
    async fn get_movie(&self, id: MediaId) -> MediaResult<Option<Movie>>;

    /// Lists all movies in the library.
    async fn list_movies(&self) -> MediaResult<Vec<Movie>>;
}
