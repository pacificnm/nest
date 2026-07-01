//! Airtable Meta API client methods.

use nest_error::NestResult;
use tracing::info;

use crate::client::AirtableClient;
use crate::meta_types::AirtableBaseSchema;

impl AirtableClient {
    /// Fetches table and field metadata for the configured base.
    pub async fn get_base_schema(&self) -> NestResult<AirtableBaseSchema> {
        let operation = "get_base_schema";
        let url = format!(
            "{}/bases/{}/tables",
            self.config().meta_api_url,
            self.config().base_id
        );
        info!(base_id = %self.config().base_id, "airtable get base schema");
        self.get_json(&url, operation).await
    }
}
