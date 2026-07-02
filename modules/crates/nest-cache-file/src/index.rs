//! In-memory tag index rebuilt from on-disk metadata.

use std::collections::{HashMap, HashSet};

/// Maps tags and payload hashes for invalidation.
#[derive(Debug, Default)]
pub struct TagIndex {
    tag_to_hashes: HashMap<String, HashSet<String>>,
    hash_to_tags: HashMap<String, Vec<String>>,
}

impl TagIndex {
    /// Registers tags for one payload hash.
    pub fn register(&mut self, hash: &str, tags: &[String]) {
        self.hash_to_tags.insert(hash.to_string(), tags.to_vec());
        for tag in tags {
            self.tag_to_hashes
                .entry(tag.clone())
                .or_default()
                .insert(hash.to_string());
        }
    }

    /// Removes one payload hash from the index.
    pub fn unregister(&mut self, hash: &str) {
        if let Some(tags) = self.hash_to_tags.remove(hash) {
            for tag in tags {
                if let Some(hashes) = self.tag_to_hashes.get_mut(&tag) {
                    hashes.remove(hash);
                    if hashes.is_empty() {
                        self.tag_to_hashes.remove(&tag);
                    }
                }
            }
        }
    }

    /// Returns payload hashes tagged with `tag`.
    pub fn hashes_for_tag(&self, tag: &str) -> Vec<String> {
        self.tag_to_hashes
            .get(tag)
            .map(|hashes| hashes.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Clears the index.
    pub fn clear(&mut self) {
        self.tag_to_hashes.clear();
        self.hash_to_tags.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unregister_removes_tag_links() {
        let mut index = TagIndex::default();
        index.register("abc", &["movies".into(), "artwork".into()]);
        index.unregister("abc");
        assert!(index.hashes_for_tag("movies").is_empty());
        assert!(index.hashes_for_tag("artwork").is_empty());
    }
}
