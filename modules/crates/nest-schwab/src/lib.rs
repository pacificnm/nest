//! Charles Schwab Trader API client module for the Nest framework.
//!
//! Built on [`nest_auth_oauth_client`] for the OAuth2 authorization-code +
//! PKCE mechanics (Schwab requires an HTTPS loopback callback — see that
//! crate's `use_https_callback`) and [`nest_http_client`] for the actual
//! HTTP calls. [`SchwabConfig`] fills in Schwab's real endpoints
//! (`https://api.schwabapi.com/...`), scope, and callback requirement so
//! callers don't hand-assemble a generic `OAuthClientConfig` themselves.
//!
//! [`SchwabModule`] registers a Schwab-configured `OAuthClient`; app code
//! builds a [`SchwabClient`] once a login (or a stored [`nest_auth::Token`])
//! produces a live credential — see `client.rs` for why response bodies are
//! `serde_json::Value` rather than typed structs in this first pass.

#![deny(missing_docs)]

pub mod client;
pub mod codes;
pub mod config;
pub mod error;
pub mod module;
pub mod prelude;

pub use client::SchwabClient;
pub use config::SchwabConfig;
pub use error::{SchwabError, SchwabErrorKind, SchwabResult};
pub use module::{SchwabModule, SCHWAB_MODULE_ID};

use nest_error::NestErrorKind;
pub use nest_error::{NestError, NestResult};

impl From<SchwabError> for NestError {
    fn from(error: SchwabError) -> Self {
        let kind = match error.kind() {
            SchwabErrorKind::Config => NestErrorKind::Config,
            SchwabErrorKind::Request => NestErrorKind::Network,
            SchwabErrorKind::Parse => NestErrorKind::Data,
            SchwabErrorKind::Auth => NestErrorKind::Auth,
            SchwabErrorKind::NotFound => NestErrorKind::Data,
        };

        NestError::new(kind, error.message())
            .with_code(error.nest_code())
            .with_module("nest-schwab")
            .with_source(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schwab_error_converts_to_nest_error() {
        let err = SchwabError::auth("token expired");
        let nest_error: NestError = err.into();
        assert_eq!(nest_error.kind(), NestErrorKind::Auth);
        assert_eq!(nest_error.code(), Some(codes::NEST_SCHWAB_AUTH_FAILED));
    }
}
