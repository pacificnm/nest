//! Cache key type.

use std::fmt;

/// Opaque namespaced cache key.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey(String);

impl CacheKey {
    /// Creates a cache key from a namespaced string.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Builds a scoped key from namespace segments.
    pub fn scoped(namespace: &str, parts: &[&str]) -> Self {
        let mut value = namespace.to_string();
        for part in parts {
            value.push(':');
            value.push_str(part);
        }
        Self(value)
    }

    /// Returns the key string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for CacheKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl From<String> for CacheKey {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scoped_key_joins_segments() {
        assert_eq!(
            CacheKey::scoped("loon", &["artwork", "alien-1979", "poster"]).as_str(),
            "loon:artwork:alien-1979:poster"
        );
    }
}
