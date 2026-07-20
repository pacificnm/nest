//! Common imports for `nest-auth-oauth-client` consumers.

pub use crate::client::{AuthorizationRequest, OAuthClient};
pub use crate::config::OAuthClientConfig;
pub use crate::error::{OAuthError, OAuthErrorKind, OAuthResult};
pub use crate::module::{OAuthClientModule, OAUTH_CLIENT_MODULE_ID};
pub use crate::token_auth::OAuthTokenAuth;
