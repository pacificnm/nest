//! Token storage.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

use async_trait::async_trait;
use tracing::{debug, warn};

use crate::error::{AuthError, AuthResult};
use crate::token::Token;

/// Persists and retrieves [`Token`]s by a caller-chosen key (e.g.
/// `"schwab"`, or a provider+account-scoped key if a caller ever needs more
/// than one credential of the same kind at once).
#[async_trait]
pub trait TokenStore: Send + Sync {
    /// Returns the stored token for `key`, or `None` if nothing is stored there.
    async fn get(&self, key: &str) -> AuthResult<Option<Token>>;

    /// Stores (overwriting any existing) token for `key`.
    async fn put(&self, key: &str, token: &Token) -> AuthResult<()>;

    /// Removes the stored token for `key`, if any. Not an error if nothing
    /// was stored there.
    async fn delete(&self, key: &str) -> AuthResult<()>;
}

/// A [`TokenStore`] backed by a single JSON file on disk.
///
/// This is the reference/dev implementation, not the recommended answer for
/// real credentials — an OS-keyring-backed store is the intended production
/// path (see `docs/nest-auth/plan.md`'s open questions). It exists so
/// `nest-auth-oauth-client` and its consumers have something concrete to
/// build and test against before that lands.
pub struct FileTokenStore {
    path: PathBuf,
    // A single in-process lock around the whole file: token acquisition is
    // an infrequent, human-triggered or refresh-interval operation, not a
    // hot path, so "read-modify-write the whole file under one lock" is
    // simple and sufficient — no need for anything fancier. Held only
    // across synchronous `std::fs` calls, never across an `.await`, so a
    // plain `std::sync::RwLock` (not an async-aware one) is fine here.
    lock: RwLock<()>,
}

impl FileTokenStore {
    /// Creates a store backed by `path`. Neither `path` nor its parent
    /// directory needs to exist yet — both are created on first
    /// [`TokenStore::put`] call, not here, so constructing a store that's
    /// never written to doesn't touch disk.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            lock: RwLock::new(()),
        }
    }

    fn read_all(&self) -> AuthResult<HashMap<String, Token>> {
        if !self.path.exists() {
            return Ok(HashMap::new());
        }
        let content = std::fs::read_to_string(&self.path).map_err(|err| {
            AuthError::io(format!("failed to read {}: {err}", self.path.display())).with_source(err)
        })?;
        if content.trim().is_empty() {
            return Ok(HashMap::new());
        }
        serde_json::from_str(&content).map_err(|err| {
            AuthError::serialize(format!("failed to parse {}: {err}", self.path.display()))
                .with_source(err)
        })
    }

    fn write_all(&self, tokens: &HashMap<String, Token>) -> AuthResult<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).map_err(|err| {
                AuthError::io(format!("failed to create {}: {err}", parent.display()))
                    .with_source(err)
            })?;
        }
        let content = serde_json::to_string_pretty(tokens).map_err(|err| {
            AuthError::serialize(format!("failed to serialize tokens: {err}")).with_source(err)
        })?;
        std::fs::write(&self.path, content).map_err(|err| {
            AuthError::io(format!("failed to write {}: {err}", self.path.display()))
                .with_source(err)
        })
    }
}

#[async_trait]
impl TokenStore for FileTokenStore {
    async fn get(&self, key: &str) -> AuthResult<Option<Token>> {
        let _guard = self.lock.read().expect("token store lock poisoned");
        let tokens = match self.read_all() {
            Ok(tokens) => tokens,
            Err(error) => {
                warn!(key = %key, error = %error, "failed to read token store");
                return Err(error);
            }
        };
        let token = tokens.get(key).cloned();
        debug!(key = %key, found = token.is_some(), "token store get");
        Ok(token)
    }

    async fn put(&self, key: &str, token: &Token) -> AuthResult<()> {
        let _guard = self.lock.write().expect("token store lock poisoned");
        let mut tokens = self.read_all()?;
        tokens.insert(key.to_string(), token.clone());
        if let Err(error) = self.write_all(&tokens) {
            warn!(key = %key, error = %error, "failed to write token store");
            return Err(error);
        }
        debug!(key = %key, "token store put");
        Ok(())
    }

    async fn delete(&self, key: &str) -> AuthResult<()> {
        let _guard = self.lock.write().expect("token store lock poisoned");
        let mut tokens = self.read_all()?;
        tokens.remove(key);
        if let Err(error) = self.write_all(&tokens) {
            warn!(key = %key, error = %error, "failed to write token store");
            return Err(error);
        }
        debug!(key = %key, "token store delete");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch_path(test_name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "nest-auth-test-{test_name}-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock")
                .as_nanos()
        ))
    }

    #[tokio::test]
    async fn get_on_a_store_that_was_never_written_to_returns_none() {
        let store = FileTokenStore::new(scratch_path("never-written"));

        assert_eq!(store.get("schwab").await.expect("get"), None);
    }

    #[tokio::test]
    async fn put_then_get_round_trips_the_token() {
        let path = scratch_path("round-trip");
        let store = FileTokenStore::new(&path);
        let token = Token::new("access-value").with_refresh_token("refresh-value");

        store.put("schwab", &token).await.expect("put");
        let fetched = store.get("schwab").await.expect("get");

        assert_eq!(fetched, Some(token));
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn put_creates_parent_directories_that_do_not_exist_yet() {
        let path = scratch_path("nested/dir/tokens").join("tokens.json");
        let store = FileTokenStore::new(&path);
        let token = Token::new("access-value");

        store.put("schwab", &token).await.expect("put");

        assert!(path.exists());
        let _ = std::fs::remove_dir_all(path.parent().expect("parent"));
    }

    #[tokio::test]
    async fn put_overwrites_an_existing_token_under_the_same_key() {
        let path = scratch_path("overwrite");
        let store = FileTokenStore::new(&path);

        store
            .put("schwab", &Token::new("first-value"))
            .await
            .expect("first put");
        store
            .put("schwab", &Token::new("second-value"))
            .await
            .expect("second put");

        let fetched = store.get("schwab").await.expect("get");
        assert_eq!(fetched, Some(Token::new("second-value")));
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn different_keys_do_not_collide() {
        let path = scratch_path("multi-key");
        let store = FileTokenStore::new(&path);

        store
            .put("schwab", &Token::new("schwab-value"))
            .await
            .expect("put schwab");
        store
            .put("other-provider", &Token::new("other-value"))
            .await
            .expect("put other");

        assert_eq!(
            store.get("schwab").await.expect("get schwab"),
            Some(Token::new("schwab-value"))
        );
        assert_eq!(
            store.get("other-provider").await.expect("get other"),
            Some(Token::new("other-value"))
        );
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn delete_removes_the_token_and_is_not_an_error_if_nothing_was_stored() {
        let path = scratch_path("delete");
        let store = FileTokenStore::new(&path);

        store
            .put("schwab", &Token::new("access-value"))
            .await
            .expect("put");
        store.delete("schwab").await.expect("delete existing");
        assert_eq!(store.get("schwab").await.expect("get after delete"), None);

        // Deleting again (nothing left to delete) must not error.
        store.delete("schwab").await.expect("delete already-gone");
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn a_second_store_pointed_at_the_same_path_sees_what_the_first_wrote() {
        let path = scratch_path("shared-path");
        let writer = FileTokenStore::new(&path);
        let reader = FileTokenStore::new(&path);

        writer
            .put("schwab", &Token::new("access-value"))
            .await
            .expect("put");

        assert_eq!(
            reader.get("schwab").await.expect("get from second store"),
            Some(Token::new("access-value"))
        );
        let _ = std::fs::remove_file(&path);
    }
}
