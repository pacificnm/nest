//! Example typed API client.

use nest_error::NestResult;
use serde::Deserialize;

use crate::HttpClientService;

/// Example customer record from an external API.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Customer {
    /// Customer id.
    pub id: String,
    /// Customer name.
    pub name: String,
}

/// Example typed API client wrapping [`HttpClientService`].
#[derive(Clone)]
pub struct ExampleApiClient {
    http: HttpClientService,
    base_url: String,
}

impl ExampleApiClient {
    /// Creates a client for the given API base URL.
    pub fn new(http: HttpClientService, base_url: impl Into<String>) -> Self {
        Self {
            http,
            base_url: base_url.into().trim_end_matches('/').to_string(),
        }
    }

    /// Fetches a customer by id.
    pub async fn get_customer(&self, id: &str) -> NestResult<Customer> {
        self.http
            .get_json(&format!("{}/customers/{id}", self.base_url))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HttpClientModule;
    use nest_core::AppBuilder;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn get_customer_round_trip() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/customers/123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "123",
                "name": "Ada"
            })))
            .mount(&server)
            .await;

        let built = AppBuilder::new()
            .module(HttpClientModule::default())
            .build()
            .unwrap();
        let http = built
            .context
            .service::<HttpClientService>()
            .unwrap()
            .clone();
        let client = ExampleApiClient::new(http, server.uri());

        let customer = client.get_customer("123").await.unwrap();
        assert_eq!(customer.id, "123");
        assert_eq!(customer.name, "Ada");
    }
}
