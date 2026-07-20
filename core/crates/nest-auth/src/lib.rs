//! Authentication token and credential-storage contracts for the Nest framework.
//!
//! nest-auth defines the mechanism-agnostic vocabulary: a [`Token`] and
//! somewhere to keep it ([`TokenStore`]). It deliberately does not define a
//! unified "auth provider" trait — acquiring a token from someone else's
//! OAuth server, issuing your own tokens, and verifying a local password are
//! different enough operations that provider crates (`nest-auth-oauth-client`,
//! and future oauth-server/password crates) build their own flow on top of
//! this shared storage layer, rather than being forced through one
//! interface. See `docs/nest-auth/plan.md` for the full reasoning.
//!
//! ```ignore
//! use nest_auth::{FileTokenStore, Token, TokenStore};
//!
//! let store = FileTokenStore::new("~/.config/finch/tokens.json");
//! let token = Token::new("access-token-value").with_refresh_token("refresh-token-value");
//! store.put("schwab", &token).await?;
//! ```

#![deny(missing_docs)]

pub mod codes;
pub mod error;
pub mod prelude;
pub mod store;
pub mod token;

pub use error::{AuthError, AuthErrorKind, AuthResult};
pub use store::{FileTokenStore, TokenStore};
pub use token::Token;
