//! Async HTTP client service.

use std::pin::Pin;
use std::time::{Duration, Instant};

use bytes::Bytes;
use futures_util::Stream;
use futures_util::StreamExt;
use nest_error::NestResult;
use nest_http::{HttpError, HttpRequest, HttpResponse, RetryPolicy};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tracing::{info, warn};

use crate::adapter::{
    build_reqwest_request, ensure_success, map_reqwest_error, map_reqwest_response,
};
use crate::config::HttpClientConfig;
use crate::http_error_to_nest_error;

/// Byte stream returned from [`HttpClientService::post_json_stream`].
pub type ByteStream = Pin<Box<dyn Stream<Item = NestResult<Bytes>> + Send>>;

/// Async HTTP client registered via [`crate::HttpClientModule`].
///
/// Provides futures only — the host must run a Tokio runtime to `.await` calls.
#[derive(Clone)]
pub struct HttpClientService {
    client: reqwest::Client,
    stream_client: reqwest::Client,
    config: HttpClientConfig,
}

impl HttpClientService {
    /// Creates a client from configuration.
    pub fn new(config: HttpClientConfig) -> NestResult<Self> {
        let mut builder = reqwest::Client::builder()
            .connect_timeout(config.default_timeout.connect)
            .timeout(config.default_timeout.request);

        if let Some(user_agent) = &config.user_agent {
            builder = builder.user_agent(user_agent);
        }

        let client = builder.build().map_err(|error| {
            nest_error::NestError::network(error.to_string()).with_source(error)
        })?;

        let mut stream_builder =
            reqwest::Client::builder().connect_timeout(config.default_timeout.connect);
        if let Some(user_agent) = &config.user_agent {
            stream_builder = stream_builder.user_agent(user_agent);
        }
        let stream_client = stream_builder.build().map_err(|error| {
            nest_error::NestError::network(error.to_string()).with_source(error)
        })?;

        Ok(Self {
            client,
            stream_client,
            config,
        })
    }

    /// Sends a GET request and decodes JSON.
    pub async fn get_json<T>(&self, url: &str) -> NestResult<T>
    where
        T: DeserializeOwned,
    {
        let response = self.send(HttpRequest::get(url)).await?;
        self.decode_json(&response, url)
    }

    /// Sends a POST request with a JSON body and decodes JSON.
    pub async fn post_json<T, B>(&self, url: &str, body: &B) -> NestResult<T>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let json = serde_json::to_vec(body).map_err(|error| {
            http_error_to_nest_error(HttpError::decode(format!(
                "failed to encode request body: {error}"
            )))
        })?;
        let request = HttpRequest::post(url)
            .with_header("content-type", "application/json")
            .with_body(json);
        let response = self.send(request).await?;
        self.decode_json(&response, url)
    }

    /// Sends a POST request with a JSON body and returns the response body stream.
    ///
    /// Streaming requests disable the per-request timeout so long-lived SSE/NDJSON
    /// responses are not cut off prematurely.
    pub async fn post_json_stream<B>(&self, url: &str, body: &B) -> NestResult<ByteStream>
    where
        B: Serialize,
    {
        let json = serde_json::to_vec(body).map_err(|error| {
            http_error_to_nest_error(HttpError::decode(format!(
                "failed to encode request body: {error}"
            )))
        })?;
        let mut request = HttpRequest::post(url)
            .with_header("content-type", "application/json")
            .with_body(json);

        if let Some(auth) = &self.config.auth {
            auth.apply(&mut request).map_err(http_error_to_nest_error)?;
        }

        self.post_stream_request(request, url).await
    }

    async fn post_stream_request(
        &self,
        request: HttpRequest,
        url: &str,
    ) -> NestResult<ByteStream> {
        let builder = build_reqwest_request(
            &self.stream_client,
            request,
            &self.config.default_headers,
        )
        .map_err(|error| http_error_to_nest_error(error.with_url(url)))?;

        let response = builder
            .timeout(Duration::from_secs(3600))
            .send()
            .await
            .map_err(|error| http_error_to_nest_error(map_reqwest_error(error).with_url(url)))?
            .error_for_status()
            .map_err(|error| http_error_to_nest_error(map_reqwest_error(error).with_url(url)))?;

        let owned_url = url.to_string();
        let stream = response.bytes_stream().map(move |chunk| {
            chunk.map_err(|error| {
                http_error_to_nest_error(map_reqwest_error(error).with_url(&owned_url))
            })
        });

        Ok(Box::pin(stream))
    }

    /// Sends an HTTP request with optional retry and auth.
    pub async fn send(&self, mut request: HttpRequest) -> NestResult<HttpResponse> {
        if let Some(auth) = &self.config.auth {
            auth.apply(&mut request).map_err(http_error_to_nest_error)?;
        }

        if let Some(retry) = &self.config.retry {
            self.send_with_retry(request, retry).await
        } else {
            self.send_once(&request)
                .await
                .map_err(http_error_to_nest_error)
        }
    }

    async fn send_with_retry(
        &self,
        request: HttpRequest,
        retry: &impl RetryPolicy,
    ) -> NestResult<HttpResponse> {
        let mut attempt = 1u32;
        loop {
            match self.send_once(&request).await {
                Ok(response) => return Ok(response),
                Err(error) => {
                    if retry.should_retry(attempt, &error) {
                        let delay = retry.delay_before_retry(attempt);
                        warn!(
                            method = %request.method,
                            url = %request.url,
                            attempt = attempt,
                            delay_ms = delay.as_millis(),
                            request_id = request.request_id.as_ref().map(|id| id.as_str()).unwrap_or("-"),
                            "HTTP request retrying"
                        );
                        tokio::time::sleep(delay).await;
                        attempt += 1;
                        continue;
                    }
                    return Err(http_error_to_nest_error(error));
                }
            }
        }
    }

    async fn send_once(&self, request: &HttpRequest) -> Result<HttpResponse, HttpError> {
        let start = Instant::now();
        info!(
            method = %request.method,
            url = %request.url,
            request_id = request.request_id.as_ref().map(|id| id.as_str()).unwrap_or("-"),
            "HTTP request started"
        );

        let builder =
            build_reqwest_request(&self.client, request.clone(), &self.config.default_headers)?;

        let result = builder.send().await;

        match result {
            Ok(response) => {
                let http_response = map_reqwest_response(response).await?;
                let duration = start.elapsed();
                if http_response.status.is_success() {
                    info!(
                        method = %request.method,
                        url = %request.url,
                        status = http_response.status.code(),
                        duration_ms = duration.as_millis(),
                        request_id = request.request_id.as_ref().map(|id| id.as_str()).unwrap_or("-"),
                        "HTTP request completed"
                    );
                } else {
                    warn!(
                        method = %request.method,
                        url = %request.url,
                        status = http_response.status.code(),
                        duration_ms = duration.as_millis(),
                        request_id = request.request_id.as_ref().map(|id| id.as_str()).unwrap_or("-"),
                        "HTTP request returned error status"
                    );
                }
                ensure_success(&http_response, &request.url)?;
                Ok(http_response)
            }
            Err(error) => {
                let http_error = map_reqwest_error(error).with_url(&request.url);
                warn!(
                    method = %request.method,
                    url = %request.url,
                    error = %http_error,
                    request_id = request.request_id.as_ref().map(|id| id.as_str()).unwrap_or("-"),
                    "HTTP request failed"
                );
                Err(http_error)
            }
        }
    }

    fn decode_json<T>(&self, response: &HttpResponse, url: &str) -> NestResult<T>
    where
        T: DeserializeOwned,
    {
        serde_json::from_slice(&response.body).map_err(|error| {
            http_error_to_nest_error(
                HttpError::decode(format!("failed to decode JSON: {error}")).with_url(url),
            )
        })
    }
}
