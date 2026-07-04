//! Cache keys and tags for remote images.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use nest_cache::CacheKey;

/// Builds a stable cache key from a fully resolved image URL.
pub fn cache_key_for_url(url: &str) -> CacheKey {
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    CacheKey::scoped("nest-image", &[&hasher.finish().to_string()])
}

/// Tag used to invalidate all cached images for one movie.
pub fn movie_cache_tag(slug: &str) -> String {
    format!("movie:{slug}")
}

/// Tags for Loon movie artwork (poster / backdrop).
pub fn artwork_tags(slug: &str) -> Vec<String> {
    vec![movie_cache_tag(slug), "artwork".into()]
}

/// Tags for cast profile images tied to a movie.
pub fn profile_tags(movie_slug: &str, person_key: &str) -> Vec<String> {
    vec![
        movie_cache_tag(movie_slug),
        format!("person:{person_key}"),
        "profile".into(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_is_stable() {
        let a = cache_key_for_url("http://example/a.jpg");
        let b = cache_key_for_url("http://example/a.jpg");
        assert_eq!(a, b);
    }
}
