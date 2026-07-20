//! Schwab API client.
//!
//! Response bodies are returned as [`serde_json::Value`] rather than typed
//! structs: Schwab's actual API reference is gated behind a logged-in
//! developer account, so account/quote/transaction schemas aren't
//! independently verifiable from here. Typed request/response structs are
//! a deliberate follow-up once real sample payloads are available (see
//! `docs/nest-auth/plan.md`'s Schwab section) — building them now would
//! mean guessing field names for a real brokerage API, which isn't a risk
//! worth taking silently.
//!
//! The generic [`SchwabClient::get_trader`]/[`SchwabClient::get_market_data`]
//! /etc. methods work against any path today; the named wrappers
//! ([`SchwabClient::account_numbers`], [`SchwabClient::quotes`], ...) exist
//! for the handful of endpoints whose exact path was confirmed against
//! Schwab's own docs (via a scraped mirror; the developer-portal originals
//! are auth-gated) or a well-established third-party client:
//! `/accounts/accountNumbers`, `/accounts`, `/accounts/{accountHash}`,
//! `/accounts/{accountHash}/orders(/{orderId})`,
//! `/accounts/{accountHash}/transactions(/{transactionId})`, `/quotes`,
//! and `/chains`.

use std::sync::Arc;

use nest_auth_oauth_client::OAuthTokenAuth;
use nest_http::{HttpMethod, HttpRequest, HttpResponse};
use nest_http_client::{HttpClientConfig, HttpClientService};
use serde_json::Value;

use crate::config::SchwabConfig;
use crate::error::SchwabResult;

/// A client for Schwab's Accounts and Trading, and Market Data, APIs.
///
/// Constructed by application code once an OAuth login has produced a
/// [`nest_auth::Token`] — not registered as a service by [`crate::SchwabModule`],
/// since (mirroring `nest-auth-oauth-client`'s own `OAuthClientModule`)
/// there's nothing to authenticate with until a login actually completes.
pub struct SchwabClient {
    http: HttpClientService,
    trader_base_url: String,
    market_data_base_url: String,
}

impl SchwabClient {
    /// Builds a client that authenticates every request with `auth`
    /// (typically backed by the [`nest_auth::Token`] a completed
    /// [`nest_auth_oauth_client::OAuthClient`] login produced).
    pub fn new(config: &SchwabConfig, auth: OAuthTokenAuth) -> SchwabResult<Self> {
        let http_config = HttpClientConfig::default().with_auth(Arc::new(auth));
        let http = HttpClientService::new(http_config)?;
        Ok(Self {
            http,
            trader_base_url: config.trader_base_url.clone(),
            market_data_base_url: config.market_data_base_url.clone(),
        })
    }

    fn trader_url(&self, path: &str) -> String {
        format!("{}{path}", self.trader_base_url)
    }

    fn market_data_url(&self, path: &str) -> String {
        format!("{}{path}", self.market_data_base_url)
    }

    /// `GET` against the Accounts and Trading API, decoded as JSON.
    pub async fn get_trader(&self, path: &str) -> SchwabResult<Value> {
        Ok(self.http.get_json(&self.trader_url(path)).await?)
    }

    /// `POST` against the Accounts and Trading API with a JSON body.
    /// Returns the raw response — Schwab's order-mutation endpoints
    /// commonly respond `201 Created` with an empty body and a `Location`
    /// header rather than a JSON payload.
    pub async fn post_trader(&self, path: &str, body: &Value) -> SchwabResult<HttpResponse> {
        self.send_trader_with_body(HttpMethod::Post, path, body)
            .await
    }

    /// `PUT` against the Accounts and Trading API with a JSON body.
    pub async fn put_trader(&self, path: &str, body: &Value) -> SchwabResult<HttpResponse> {
        self.send_trader_with_body(HttpMethod::Put, path, body)
            .await
    }

    /// `DELETE` against the Accounts and Trading API.
    pub async fn delete_trader(&self, path: &str) -> SchwabResult<HttpResponse> {
        let request = HttpRequest::new(HttpMethod::Delete, self.trader_url(path));
        Ok(self.http.send(request).await?)
    }

    /// `GET` against the Market Data API, decoded as JSON. `query` pairs
    /// are appended as `?key=value&...` (values are used as-is — callers
    /// are expected to pass values that don't need percent-encoding, e.g.
    /// symbols and enum-like strings).
    pub async fn get_market_data(&self, path: &str, query: &[(&str, &str)]) -> SchwabResult<Value> {
        let mut url = self.market_data_url(path);
        append_query(&mut url, query);
        Ok(self.http.get_json(&url).await?)
    }

    async fn send_trader_with_body(
        &self,
        method: HttpMethod,
        path: &str,
        body: &Value,
    ) -> SchwabResult<HttpResponse> {
        let json = serde_json::to_vec(body).map_err(|err| {
            crate::error::SchwabError::parse(format!("failed to encode request body: {err}"))
                .with_source(err)
        })?;
        let request = HttpRequest::new(method, self.trader_url(path))
            .with_header("content-type", "application/json")
            .with_body(json);
        Ok(self.http.send(request).await?)
    }

    /// `GET /accounts/accountNumbers` — the account-number-to-hash mapping
    /// every other account-scoped call must use instead of the raw account
    /// number.
    pub async fn account_numbers(&self) -> SchwabResult<Value> {
        self.get_trader("/accounts/accountNumbers").await
    }

    /// `GET /accounts` — all linked accounts.
    pub async fn accounts(&self) -> SchwabResult<Value> {
        self.get_trader("/accounts").await
    }

    /// `GET /accounts/{account_hash}` — a single account.
    pub async fn account(&self, account_hash: &str) -> SchwabResult<Value> {
        self.get_trader(&format!("/accounts/{account_hash}")).await
    }

    /// `GET /accounts/{account_hash}/orders` — orders for an account.
    pub async fn orders_for_account(&self, account_hash: &str) -> SchwabResult<Value> {
        self.get_trader(&format!("/accounts/{account_hash}/orders"))
            .await
    }

    /// `GET /accounts/{account_hash}/orders/{order_id}` — a single order.
    pub async fn order(&self, account_hash: &str, order_id: &str) -> SchwabResult<Value> {
        self.get_trader(&format!("/accounts/{account_hash}/orders/{order_id}"))
            .await
    }

    /// `POST /accounts/{account_hash}/orders` — places a new order.
    pub async fn place_order(
        &self,
        account_hash: &str,
        order: &Value,
    ) -> SchwabResult<HttpResponse> {
        self.post_trader(&format!("/accounts/{account_hash}/orders"), order)
            .await
    }

    /// `PUT /accounts/{account_hash}/orders/{order_id}` — replaces an
    /// existing (not-yet-filled) order.
    pub async fn replace_order(
        &self,
        account_hash: &str,
        order_id: &str,
        order: &Value,
    ) -> SchwabResult<HttpResponse> {
        self.put_trader(
            &format!("/accounts/{account_hash}/orders/{order_id}"),
            order,
        )
        .await
    }

    /// `DELETE /accounts/{account_hash}/orders/{order_id}` — cancels an
    /// order.
    pub async fn cancel_order(
        &self,
        account_hash: &str,
        order_id: &str,
    ) -> SchwabResult<HttpResponse> {
        self.delete_trader(&format!("/accounts/{account_hash}/orders/{order_id}"))
            .await
    }

    /// `GET /accounts/{account_hash}/transactions` — transactions for an account.
    pub async fn transactions(&self, account_hash: &str) -> SchwabResult<Value> {
        self.get_trader(&format!("/accounts/{account_hash}/transactions"))
            .await
    }

    /// `GET /accounts/{account_hash}/transactions/{transaction_id}` — a
    /// single transaction.
    pub async fn transaction(
        &self,
        account_hash: &str,
        transaction_id: &str,
    ) -> SchwabResult<Value> {
        self.get_trader(&format!(
            "/accounts/{account_hash}/transactions/{transaction_id}"
        ))
        .await
    }

    /// `GET /quotes?symbols=...` — quotes for one or more symbols.
    pub async fn quotes(&self, symbols: &[&str]) -> SchwabResult<Value> {
        let joined = symbols.join(",");
        self.get_market_data("/quotes", &[("symbols", joined.as_str())])
            .await
    }

    /// `GET /chains?symbol=...` — the option chain for a symbol.
    pub async fn option_chain(&self, symbol: &str) -> SchwabResult<Value> {
        self.get_market_data("/chains", &[("symbol", symbol)]).await
    }
}

fn append_query(url: &mut String, query: &[(&str, &str)]) {
    if query.is_empty() {
        return;
    }
    url.push('?');
    for (index, (key, value)) in query.iter().enumerate() {
        if index > 0 {
            url.push('&');
        }
        url.push_str(key);
        url.push('=');
        url.push_str(value);
    }
}

#[cfg(test)]
mod tests {
    use nest_auth::Token;
    use serde_json::json;
    use wiremock::matchers::{body_json, header, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;
    use crate::config::SchwabConfig;

    async fn client_for(server: &MockServer) -> SchwabClient {
        let config = SchwabConfig::new("app-key", "app-secret")
            .with_trader_base_url(server.uri())
            .with_market_data_base_url(server.uri());
        let auth = OAuthTokenAuth::new(Token::new("test-access-token"));
        SchwabClient::new(&config, auth).expect("client")
    }

    #[tokio::test]
    async fn account_numbers_hits_the_expected_path_with_bearer_auth() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/accounts/accountNumbers"))
            .and(header("authorization", "Bearer test-access-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([
                {"accountNumber": "123456789", "hashValue": "ABC123HASH"}
            ])))
            .mount(&server)
            .await;

        let client = client_for(&server).await;
        let response = client.account_numbers().await.expect("account_numbers");

        assert_eq!(response[0]["hashValue"], "ABC123HASH");
    }

    #[tokio::test]
    async fn quotes_sends_symbols_as_a_comma_joined_query_param() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/quotes"))
            .and(query_param("symbols", "AAPL,MSFT"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"AAPL": {}, "MSFT": {}})))
            .mount(&server)
            .await;

        let client = client_for(&server).await;
        let response = client.quotes(&["AAPL", "MSFT"]).await.expect("quotes");

        assert!(response.get("AAPL").is_some());
    }

    #[tokio::test]
    async fn place_order_posts_the_order_body_to_the_account_orders_path() {
        let server = MockServer::start().await;
        let order = json!({"orderType": "MARKET", "session": "NORMAL"});
        Mock::given(method("POST"))
            .and(path("/accounts/ABC123HASH/orders"))
            .and(body_json(&order))
            .respond_with(
                ResponseTemplate::new(201)
                    .insert_header("location", "/accounts/ABC123HASH/orders/999"),
            )
            .mount(&server)
            .await;

        let client = client_for(&server).await;
        let response = client
            .place_order("ABC123HASH", &order)
            .await
            .expect("place_order");

        assert_eq!(response.status.code(), 201);
    }

    #[tokio::test]
    async fn cancel_order_sends_delete_to_the_order_path() {
        let server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/accounts/ABC123HASH/orders/999"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let client = client_for(&server).await;
        client
            .cancel_order("ABC123HASH", "999")
            .await
            .expect("cancel_order");
    }
}
