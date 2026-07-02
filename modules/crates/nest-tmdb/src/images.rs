//! TMDB image URL construction.

use std::sync::Arc;

use nest_media::{Artwork, ArtworkKind, ArtworkSource};
use tokio::sync::RwLock;

use crate::config::DEFAULT_IMAGE_BASE_URL;

/// TMDB image size tokens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageSize {
    /// 92px wide poster.
    W92,
    /// 154px wide poster.
    W154,
    /// 342px wide poster.
    W342,
    /// 500px wide poster.
    W500,
    /// 780px wide poster.
    W780,
    /// Original image dimensions.
    Original,
    /// 300px wide backdrop.
    W300,
    /// 1280px wide backdrop.
    W1280,
    /// 185px wide profile image.
    W185,
}

impl ImageSize {
    fn as_str(self) -> &'static str {
        match self {
            Self::W92 => "w92",
            Self::W154 => "w154",
            Self::W342 => "w342",
            Self::W500 => "w500",
            Self::W780 => "w780",
            Self::Original => "original",
            Self::W300 => "w300",
            Self::W1280 => "w1280",
            Self::W185 => "w185",
        }
    }
}

/// Builds TMDB image URLs from configuration and file paths.
#[derive(Clone)]
pub struct TmdbImageService {
    base_url: Arc<RwLock<String>>,
}

impl TmdbImageService {
    /// Creates an image service backed by a shared base URL.
    pub fn new(base_url: Arc<RwLock<String>>) -> Self {
        Self { base_url }
    }

    /// Returns the current image base URL.
    pub async fn base_url(&self) -> String {
        self.base_url.read().await.clone()
    }

    /// Builds a poster image URL.
    pub fn poster_url_with_base(base_url: &str, path: &str, size: ImageSize) -> String {
        join_image_url(base_url, size.as_str(), path)
    }

    /// Builds a backdrop image URL.
    pub fn backdrop_url_with_base(base_url: &str, path: &str, size: ImageSize) -> String {
        join_image_url(base_url, size.as_str(), path)
    }

    /// Builds a poster image URL using the configured base URL.
    pub async fn poster_url(&self, path: &str, size: ImageSize) -> String {
        let base = self.base_url.read().await;
        Self::poster_url_with_base(&base, path, size)
    }

    /// Builds a backdrop image URL using the configured base URL.
    pub async fn backdrop_url(&self, path: &str, size: ImageSize) -> String {
        let base = self.base_url.read().await;
        Self::backdrop_url_with_base(&base, path, size)
    }

    /// Builds a profile image URL.
    pub fn profile_url_with_base(base_url: &str, path: &str, size: ImageSize) -> String {
        join_image_url(base_url, size.as_str(), path)
    }

    /// Builds nest-media artwork entries for a movie's primary poster and backdrop.
    pub async fn artwork_for_movie(
        &self,
        poster_path: Option<&str>,
        backdrop_path: Option<&str>,
    ) -> Vec<Artwork> {
        let base = self.base_url.read().await;
        artwork_for_movie_with_base(&base, poster_path, backdrop_path)
    }
}

/// Builds artwork using a known image base URL (sync helper for tests and mapping).
pub fn artwork_for_movie_with_base(
    base_url: &str,
    poster_path: Option<&str>,
    backdrop_path: Option<&str>,
) -> Vec<Artwork> {
    let mut artwork = Vec::new();

    if let Some(path) = poster_path.filter(|value| !value.is_empty()) {
        artwork.push(Artwork {
            kind: ArtworkKind::Poster,
            source: ArtworkSource::RemoteUrl(TmdbImageService::poster_url_with_base(
                base_url,
                path,
                ImageSize::W500,
            )),
            width: None,
            height: None,
        });
    }

    if let Some(path) = backdrop_path.filter(|value| !value.is_empty()) {
        artwork.push(Artwork {
            kind: ArtworkKind::Backdrop,
            source: ArtworkSource::RemoteUrl(TmdbImageService::backdrop_url_with_base(
                base_url,
                path,
                ImageSize::W1280,
            )),
            width: None,
            height: None,
        });
    }

    artwork
}

fn join_image_url(base_url: &str, size: &str, path: &str) -> String {
    let base = if base_url.ends_with('/') {
        base_url.to_string()
    } else {
        format!("{base_url}/")
    };
    let path = path.trim_start_matches('/');
    format!("{base}{size}/{path}")
}

impl Default for TmdbImageService {
    fn default() -> Self {
        Self::new(Arc::new(RwLock::new(DEFAULT_IMAGE_BASE_URL.to_string())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_poster_url() {
        let url = TmdbImageService::poster_url_with_base(
            "https://image.tmdb.org/t/p/",
            "/abc.jpg",
            ImageSize::W500,
        );
        assert_eq!(url, "https://image.tmdb.org/t/p/w500/abc.jpg");
    }

    #[tokio::test]
    async fn builds_artwork_for_movie() {
        let service = TmdbImageService::default();
        let artwork = service
            .artwork_for_movie(Some("/poster.jpg"), Some("/backdrop.jpg"))
            .await;
        assert_eq!(artwork.len(), 2);
        assert_eq!(artwork[0].kind, ArtworkKind::Poster);
        assert_eq!(artwork[1].kind, ArtworkKind::Backdrop);
    }
}
