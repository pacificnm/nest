//! Async HTTP client for consuming external APIs in the Nest framework.
//!
//! Wraps reqwest behind [`HttpClientService`] and registers it via
//! [`HttpClientModule`]. Async modules provide futures; the host owns the
//! Tokio runtime.

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

mod adapter;
mod config;
mod example;
mod module;
mod prelude;
mod service;

pub use config::HttpClientConfig;
pub use example::{Customer, ExampleApiClient};
pub use module::{HttpClientModule, HTTP_CLIENT_MODULE_ID};
pub use service::HttpClientService;

pub use nest_core::{Module, ModuleId};
pub use nest_error::{NestError, NestResult};
pub use nest_http::{
    AuthStrategy, BearerTokenAuth, FixedRetryPolicy, HttpError, HttpMethod, HttpRequest,
    HttpResponse, HttpStatus, RequestId, TimeoutConfig,
};

/// Converts an [`HttpError`] into a [`NestError`].
pub fn http_error_to_nest_error(error: HttpError) -> NestError {
    let mut nest_error = NestError::network(error.message())
        .with_code(error.nest_code())
        .with_module("nest-http-client");

    if let Some(url) = error.url() {
        nest_error = nest_error.with_operation(format!("url: {url}"));
    }
    if let Some(status) = error.response_status() {
        nest_error = nest_error.with_help(format!("HTTP status: {}", status.code()));
    }

    nest_error.with_source(error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nest_error::NestErrorKind;
    use std::sync::Arc;
    use std::time::Duration;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use nest_core::AppBuilder;

    #[tokio::test]
    async fn get_json_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/hello"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "message": "world"
            })))
            .mount(&server)
            .await;

        let built = AppBuilder::new()
            .module(HttpClientModule::default())
            .build()
            .unwrap();
        let http = built.context.service::<HttpClientService>().unwrap();

        #[derive(serde::Deserialize)]
        struct Payload {
            message: String,
        }

        let payload: Payload = http
            .get_json(&format!("{}/hello", server.uri()))
            .await
            .unwrap();
        assert_eq!(payload.message, "world");
    }

    #[tokio::test]
    async fn post_json_success() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/items"))
            .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({
                "id": 1
            })))
            .mount(&server)
            .await;

        let built = AppBuilder::new()
            .module(HttpClientModule::default())
            .build()
            .unwrap();
        let http = built.context.service::<HttpClientService>().unwrap();

        #[derive(serde::Serialize)]
        struct CreateItem {
            name: String,
        }
        #[derive(serde::Deserialize)]
        struct Item {
            id: u32,
        }

        let item: Item = http
            .post_json(
                &format!("{}/items", server.uri()),
                &CreateItem {
                    name: "test".into(),
                },
            )
            .await
            .unwrap();
        assert_eq!(item.id, 1);
    }

    #[tokio::test]
    async fn decode_error_maps_to_nest_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/bad"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
            .mount(&server)
            .await;

        let built = AppBuilder::new()
            .module(HttpClientModule::default())
            .build()
            .unwrap();
        let http = built.context.service::<HttpClientService>().unwrap();

        #[derive(serde::Deserialize)]
        struct Payload {
            message: String,
        }

        let result = http
            .get_json::<Payload>(&format!("{}/bad", server.uri()))
            .await;
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(err.kind(), NestErrorKind::Network);
        assert_eq!(err.code(), Some(nest_http::codes::NEST_HTTP_DECODE_FAILED));
    }

    #[tokio::test]
    async fn retry_on_503_then_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/flaky"))
            .respond_with(ResponseTemplate::new(503))
            .up_to_n_times(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/flaky"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "ok": true
            })))
            .mount(&server)
            .await;

        let config = HttpClientConfig::default().with_retry(FixedRetryPolicy::new(
            3,
            Duration::from_millis(10),
        ));
        let http = HttpClientService::new(config).unwrap();

        #[derive(serde::Deserialize)]
        struct Payload {
            ok: bool,
        }

        let payload: Payload = http
            .get_json(&format!("{}/flaky", server.uri()))
            .await
            .unwrap();
        assert!(payload.ok);
    }

    #[tokio::test]
    async fn bearer_auth_header_sent() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/secure"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&server)
            .await;

        let config = HttpClientConfig::default()
            .with_auth(Arc::new(BearerTokenAuth::new("test-token")));
        let http = HttpClientService::new(config).unwrap();
        let _: serde_json::Value = http
            .get_json(&format!("{}/secure", server.uri()))
            .await
            .unwrap();
    }

    #[test]
    fn http_error_converts_to_nest_error() {
        let err = HttpError::timeout("timed out");
        let nest_error = http_error_to_nest_error(err);
        assert_eq!(nest_error.kind(), NestErrorKind::Network);
        assert_eq!(nest_error.code(), Some(nest_http::codes::NEST_HTTP_TIMEOUT));
    }
}
