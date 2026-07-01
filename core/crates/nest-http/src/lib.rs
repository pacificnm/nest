//! Shared HTTP contracts for the Nest framework.
//!
//! nest-http defines the shared HTTP language: methods, status codes, headers,
//! request/response types, auth/retry contracts, and errors. It does not perform
//! networking — see `nest-http-client` and `nest-http-serve`.

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

pub mod auth;
pub mod codes;
mod correlation;
mod error;
mod headers;
mod method;
mod pagination;
pub mod prelude;
mod request;
mod response;
mod retry;
mod status;
mod timeout;

pub use auth::{AuthStrategy, BearerTokenAuth};
pub use correlation::{CorrelationId, RequestId};
pub use error::{HttpError, HttpErrorKind, HttpResult};
pub use headers::HeaderMap;
pub use method::HttpMethod;
pub use pagination::{Page, PageRequest};
pub use request::HttpRequest;
pub use response::{ApiResponse, HttpResponse};
pub use retry::{FixedRetryPolicy, RetryPolicy};
pub use status::HttpStatus;
pub use timeout::TimeoutConfig;

#[cfg(feature = "serde")]
pub use serde_json;
