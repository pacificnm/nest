//! HTTP header map helpers.

use std::collections::HashMap;

/// Case-insensitive HTTP header map (stored with lowercase keys).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HeaderMap {
    headers: HashMap<String, String>,
}

impl HeaderMap {
    /// Creates an empty header map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a header value.
    pub fn insert(&mut self, name: impl AsRef<str>, value: impl Into<String>) {
        self.headers
            .insert(name.as_ref().to_ascii_lowercase(), value.into());
    }

    /// Returns a header value by name.
    pub fn get(&self, name: impl AsRef<str>) -> Option<&str> {
        self.headers
            .get(&name.as_ref().to_ascii_lowercase())
            .map(String::as_str)
    }

    /// Returns an iterator over header name/value pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.headers.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }

    /// Returns whether the map is empty.
    pub fn is_empty(&self) -> bool {
        self.headers.is_empty()
    }
}
