//! Airtable REST API client.

use nest_error::NestResult;
use nest_http::{HttpError, HttpMethod, HttpRequest, HttpResponse, RetryPolicy};
use nest_http_client::HttpClientService;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tracing::{info, warn};

use crate::batch::AirtableBatch;
use crate::config::AirtableConfig;
use crate::error::{check_cancelled, http_to_airtable_error, invalid_response};
use crate::retry::{AirtableRateLimitHook, AirtableRetryPolicy};
use crate::types::{
    AirtableBatchUpdate, AirtableListPage, AirtableListParams, AirtableRecord, BatchUpdateResponse,
};

/// Airtable REST API client registered via [`crate::AirtableModule`].
#[derive(Clone)]
pub struct AirtableClient {
    http: HttpClientService,
    config: AirtableConfig,
    retry: AirtableRetryPolicy,
    rate_limit: AirtableRateLimitHook,
}

impl AirtableClient {
    /// Creates a client with default retry and rate-limit hooks.
    pub fn new(http: HttpClientService, config: AirtableConfig) -> NestResult<Self> {
        Ok(Self {
            http,
            config,
            retry: AirtableRetryPolicy::default(),
            rate_limit: AirtableRateLimitHook::default(),
        })
    }

    /// Sets the retry policy.
    pub fn with_retry(mut self, retry: AirtableRetryPolicy) -> Self {
        self.retry = retry;
        self
    }

    /// Sets the inter-page rate-limit hook.
    pub fn with_rate_limit(mut self, rate_limit: AirtableRateLimitHook) -> Self {
        self.rate_limit = rate_limit;
        self
    }

    /// Returns the resolved configuration.
    pub fn config(&self) -> &AirtableConfig {
        &self.config
    }

    /// Lists one page of records.
    pub async fn list_records_page(
        &self,
        table: &str,
        params: &AirtableListParams,
    ) -> NestResult<AirtableListPage> {
        self.list_records_page_with_offset(table, params, None)
            .await
    }

    /// Lists one page using an explicit offset token.
    pub async fn list_records_page_with_offset(
        &self,
        table: &str,
        params: &AirtableListParams,
        offset: Option<&str>,
    ) -> NestResult<AirtableListPage> {
        check_cancelled(params.cancel.as_ref())?;
        let operation = format!("list_records:{table}");
        let url = self.build_list_url(table, params, offset)?;
        info!(table, "airtable list records page");
        self.get_json(&url, &operation).await
    }

    /// Lists all records, following offset pagination until exhausted.
    pub async fn list_all_records(
        &self,
        table: &str,
        params: AirtableListParams,
    ) -> NestResult<Vec<AirtableRecord>> {
        let mut pager = crate::pager::AirtablePager::new(self.clone(), table, params);
        let mut records = Vec::new();

        while let Some(page) = pager.next_page().await? {
            records.extend(page.records);
            if page.offset.is_some() {
                self.rate_limit.after_page().await;
            }
        }

        Ok(records)
    }

    /// Updates a single record by id.
    pub async fn update_record(
        &self,
        table: &str,
        record_id: &str,
        fields: crate::types::AirtableFields,
    ) -> NestResult<AirtableRecord> {
        let operation = format!("update_record:{table}:{record_id}");
        let url = self.record_url(table, record_id)?;
        info!(table, record_id, "airtable update record");
        #[derive(Serialize)]
        struct Body {
            fields: crate::types::AirtableFields,
        }
        self.patch_json(&url, &Body { fields }, &operation).await
    }

    /// Updates records, chunking into Airtable batches of 10 when needed.
    pub async fn batch_update_records(
        &self,
        table: &str,
        updates: Vec<AirtableBatchUpdate>,
    ) -> NestResult<Vec<AirtableRecord>> {
        if updates.is_empty() {
            return Ok(Vec::new());
        }

        let mut all_records = Vec::with_capacity(updates.len());
        for chunk in AirtableBatch::chunk_updates(updates) {
            let mut page = self.batch_update_records_page(table, chunk).await?;
            all_records.append(&mut page);
            self.rate_limit.after_page().await;
        }
        Ok(all_records)
    }

    async fn batch_update_records_page(
        &self,
        table: &str,
        updates: Vec<AirtableBatchUpdate>,
    ) -> NestResult<Vec<AirtableRecord>> {
        let operation = format!("batch_update_records:{table}");
        let url = self.table_url(table)?;
        let batch = AirtableBatch::updates(updates)?;
        info!(table, count = batch.len(), "airtable batch update records");

        #[derive(Serialize)]
        struct Body<'a> {
            records: &'a [AirtableBatchUpdate],
        }

        let records = batch.into_items();
        let response: BatchUpdateResponse = self
            .patch_json(&url, &Body { records: &records }, &operation)
            .await?;
        Ok(response.records)
    }

    fn table_url(&self, table: &str) -> NestResult<String> {
        let table_cfg = self.config.table(table)?;
        Ok(format!(
            "{}/{}/{}",
            self.config.api_url, self.config.base_id, table_cfg.table_id
        ))
    }

    fn record_url(&self, table: &str, record_id: &str) -> NestResult<String> {
        Ok(format!("{}/{}", self.table_url(table)?, record_id))
    }

    fn build_list_url(
        &self,
        table: &str,
        params: &AirtableListParams,
        offset: Option<&str>,
    ) -> NestResult<String> {
        let mut url = self.table_url(table)?;
        let mut query = vec![format!("pageSize={}", params.effective_page_size())];

        if let Some(offset) = offset.filter(|value| !value.is_empty()) {
            query.push(format!("offset={}", percent_encode(offset)));
        }
        if let Some(view) = &params.view {
            query.push(format!("view={}", percent_encode(view)));
        }
        if let Some(formula) = &params.filter_by_formula {
            query.push(format!("filterByFormula={}", percent_encode(formula)));
        }
        for field in params.fields.iter().flatten() {
            query.push(format!("fields[]={}", percent_encode(field)));
        }

        url.push('?');
        url.push_str(&query.join("&"));
        Ok(url)
    }

    pub(crate) async fn get_json<T>(&self, url: &str, operation: &str) -> NestResult<T>
    where
        T: DeserializeOwned,
    {
        let request = self.authorized(HttpRequest::get(url));
        let response = self.send_airtable(request, operation).await?;
        self.decode_json(&response, url, operation)
    }

    async fn patch_json<T, B>(&self, url: &str, body: &B, operation: &str) -> NestResult<T>
    where
        T: DeserializeOwned,
        B: Serialize,
    {
        let json = serde_json::to_vec(body).map_err(|error| {
            invalid_response(format!("failed to encode request body: {error}"), operation)
        })?;
        let request = self
            .authorized(HttpRequest::new(HttpMethod::Patch, url))
            .with_header("content-type", "application/json")
            .with_body(json);
        let response = self.send_airtable(request, operation).await?;
        self.decode_json(&response, url, operation)
    }

    fn authorized(&self, request: HttpRequest) -> HttpRequest {
        request.with_header("authorization", format!("Bearer {}", self.config.token))
    }

    async fn send_airtable(
        &self,
        request: HttpRequest,
        operation: &str,
    ) -> NestResult<HttpResponse> {
        let mut attempt = 1u32;
        loop {
            match self.http.send(request.clone()).await {
                Ok(response) => return Ok(response),
                Err(error) => {
                    if let Some(http_error) = error
                        .source()
                        .and_then(|source| source.downcast_ref::<HttpError>())
                    {
                        if self.retry.should_retry(attempt, http_error) {
                            let delay = self.retry.delay_before_retry(attempt);
                            warn!(
                                attempt,
                                delay_ms = delay.as_millis(),
                                operation,
                                "airtable request retrying"
                            );
                            tokio::time::sleep(delay).await;
                            attempt += 1;
                            continue;
                        }
                    }
                    return Err(http_to_airtable_error(error, operation));
                }
            }
        }
    }

    fn decode_json<T>(&self, response: &HttpResponse, url: &str, operation: &str) -> NestResult<T>
    where
        T: DeserializeOwned,
    {
        serde_json::from_slice(&response.body).map_err(|error| {
            invalid_response(
                format!("failed to decode Airtable JSON from {url}: {error}"),
                operation,
            )
        })
    }
}

fn percent_encode(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (byte as char).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_encodes_offset_slashes() {
        assert_eq!(percent_encode("itrXXX/recYYY"), "itrXXX%2FrecYYY");
    }
}
