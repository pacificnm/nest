//! OAuth2 authorization-code + PKCE client module for the Nest framework.
//!
//! Implements the generic RFC 6749 + PKCE flow on top of the
//! [`oauth2`](https://docs.rs/oauth2) crate: builds the authorization URL,
//! runs a local loopback listener for the redirect, exchanges the code for
//! a [`nest_auth::Token`], and refreshes it later. [`OAuthTokenAuth`] then
//! bridges an acquired token into [`nest_http`]'s existing `AuthStrategy`,
//! so authenticated calls flow through the same `nest-http-client` pipeline
//! every other module already uses.
//!
//! Deliberately provider-agnostic at this layer — see `docs/nest-auth/plan.md`
//! for why Schwab-specific config/behavior stays a config concern here
//! rather than a separate crate, at least until a second OAuth provider is
//! a real requirement.

#![deny(missing_docs)]

pub mod callback;
pub mod client;
pub mod codes;
pub mod config;
pub mod error;
pub mod module;
pub mod prelude;
pub mod token_auth;

pub use client::{AuthorizationRequest, OAuthClient};
pub use config::OAuthClientConfig;
pub use error::{OAuthError, OAuthErrorKind, OAuthResult};
pub use module::{OAuthClientModule, OAUTH_CLIENT_MODULE_ID};
pub use token_auth::OAuthTokenAuth;

use nest_error::NestErrorKind;
pub use nest_error::{NestError, NestResult};

impl From<OAuthError> for NestError {
    fn from(error: OAuthError) -> Self {
        let kind = match error.kind() {
            OAuthErrorKind::Config => NestErrorKind::Config,
            OAuthErrorKind::Request => NestErrorKind::Network,
            OAuthErrorKind::Parse => NestErrorKind::Data,
            OAuthErrorKind::StateMismatch => NestErrorKind::Validation,
            OAuthErrorKind::AccessDenied => NestErrorKind::Auth,
            OAuthErrorKind::Callback => NestErrorKind::Network,
        };

        NestError::new(kind, error.message())
            .with_code(error.nest_code())
            .with_module("nest-auth-oauth-client")
            .with_source(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oauth_error_converts_to_nest_error() {
        let err = OAuthError::access_denied("user cancelled");
        let nest_error: NestError = err.into();
        assert_eq!(nest_error.kind(), NestErrorKind::Auth);
        assert_eq!(
            nest_error.code(),
            Some(codes::NEST_AUTH_OAUTH_ACCESS_DENIED)
        );
    }
}
