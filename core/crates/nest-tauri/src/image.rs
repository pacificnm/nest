//! Image fetch and cache invalidation for the React webview.

use nest_core::AppContext;
use nest_error::NestResult;
use nest_image::{cache_key_for_url, detect_mime, ImageService};

/// Request payload for [`crate::commands::nest_image_fetch`].
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImageFetchRequest {
    /// Fully resolved HTTP(S) URL to fetch.
    pub url: String,
    /// Optional cache invalidation tags (e.g. `movie:alien`, `artwork`).
    pub tags: Option<Vec<String>>,
}

/// Response payload for image fetch commands.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImageFetchResponse {
    /// Base64-encoded image bytes.
    pub bytes_base64: String,
    /// Detected MIME type (e.g. `image/jpeg`).
    pub mime: String,
    /// Stable cache key for this URL.
    pub cache_key: String,
}

/// Request payload for [`crate::commands::nest_image_invalidate_tag`].
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImageInvalidateTagRequest {
    /// Cache tag to invalidate (e.g. `movie:alien`).
    pub tag: String,
}

/// Response payload for tag invalidation commands.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ImageInvalidateTagResponse {
    /// Number of cache entries removed.
    pub removed_count: u64,
}

/// Fetches (or loads from cache) image bytes for the React webview.
pub fn fetch_image(
    context: &AppContext,
    url: &str,
    tags: &[String],
) -> NestResult<ImageFetchResponse> {
    let images = context.service::<ImageService>()?;
    let key = cache_key_for_url(url);
    let tag_refs: Vec<&str> = tags.iter().map(String::as_str).collect();
    let bytes = images.fetch_bytes(url, &key, &tag_refs)?;
    let mime = detect_mime(&bytes);
    Ok(ImageFetchResponse {
        bytes_base64: base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes),
        mime: mime.to_string(),
        cache_key: key.to_string(),
    })
}

/// Invalidates all cached images with the given tag.
pub fn invalidate_image_tag(
    context: &AppContext,
    tag: &str,
) -> NestResult<ImageInvalidateTagResponse> {
    let images = context.service::<ImageService>()?;
    let removed_count = images.invalidate_tag(tag)?;
    Ok(ImageInvalidateTagResponse { removed_count })
}

#[cfg(test)]
mod tests {
    use nest_cache::Cache;
    use nest_core::AppBuilder;
    use nest_image::ImageModule;

    use super::*;

    #[test]
    fn fetch_image_requires_image_service() {
        let built = AppBuilder::new().build().unwrap();
        let error = fetch_image(&built.context, "https://example.com/a.jpg", &[]).unwrap_err();
        assert!(error.to_string().contains("ImageService"));
    }

    #[test]
    fn fetch_image_uses_cache_without_http() {
        use nest_image::cache_key_for_url;

        let cache = Cache::memory();
        let url = "https://example.com/cached.jpg";
        let key = cache_key_for_url(url);
        cache
            .set_bytes(key.clone(), b"jpeg-bytes".to_vec(), &["artwork"], None)
            .unwrap();

        let built = AppBuilder::new()
            .module(ImageModule::with_cache(cache))
            .build()
            .unwrap();

        let response = fetch_image(&built.context, url, &[]).unwrap();
        assert_eq!(response.mime, "application/octet-stream");
        assert_eq!(response.cache_key, key.to_string());
        assert!(!response.bytes_base64.is_empty());
    }

    #[test]
    fn invalidate_tag_delegates_to_service() {
        let cache = Cache::memory();
        cache
            .set_bytes(
                nest_cache::CacheKey::new("poster"),
                b"old".to_vec(),
                &["movie:alien"],
                None,
            )
            .unwrap();

        let built = AppBuilder::new()
            .module(ImageModule::with_cache(cache))
            .build()
            .unwrap();

        let response = invalidate_image_tag(&built.context, "movie:alien").unwrap();
        assert_eq!(response.removed_count, 1);
    }
}
