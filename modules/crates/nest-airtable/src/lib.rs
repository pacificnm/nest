//! Airtable REST API integration for the Nest framework.
//!
//! Provides [`AirtableClient`] for listing and updating records with offset-based
//! pagination, batch helpers, and Airtable-specific retry handling. Uses
//! [`nest_http_client::HttpClientService`] for transport and Bearer token auth.

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

pub mod codes;
pub mod prelude;

mod batch;
mod client;
mod config;
mod error;
mod meta;
mod meta_types;
mod module;
mod pager;
mod retry;
mod types;

pub use batch::AirtableBatch;
pub use client::AirtableClient;
pub use config::{
    looks_like_secret, resolve_airtable_token, AirtableConfig, AirtableConfigBuilder,
    AirtableTableConfig, DEFAULT_META_API_URL,
};
pub use meta_types::{
    is_computed_field_type, AirtableBaseSchema, AirtableFieldSchema, AirtableTableSchema,
};
pub use module::{AirtableModule, AIRTABLE_MODULE_ID};
pub use pager::AirtablePager;
pub use retry::{AirtableRateLimitHook, AirtableRetryPolicy};
pub use types::{
    AirtableBatchUpdate, AirtableFields, AirtableListPage, AirtableListParams, AirtableRecord,
    DEFAULT_PAGE_SIZE, MAX_BATCH_UPDATE_SIZE,
};

pub use nest_core::{Module, ModuleId};
pub use nest_error::{NestError, NestResult};

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use nest_config::ConfigDocument;
    use nest_core::AppBuilder;
    use nest_http_client::HttpClientModule;
    use serde_json::json;
    use wiremock::matchers::{header, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    use super::*;
    use crate::types::AirtableFields;

    fn test_config(base_uri: &str) -> AirtableConfig {
        AirtableConfig::builder("appTEST", "pat-test")
            .api_url(base_uri)
            .table("assets", "tblASSETS", Some("Asset ID".into()))
            .build()
            .unwrap()
    }

    fn test_app(config: AirtableConfig) -> AirtableClient {
        test_app_with_rate_limit(config, AirtableRateLimitHook::default())
    }

    fn test_app_with_rate_limit(
        config: AirtableConfig,
        rate_limit: AirtableRateLimitHook,
    ) -> AirtableClient {
        let built = AppBuilder::new()
            .module(HttpClientModule::default())
            .module(AirtableModule::with_config(config))
            .build()
            .unwrap();
        built
            .context
            .service::<AirtableClient>()
            .unwrap()
            .clone()
            .with_rate_limit(rate_limit)
    }

    #[tokio::test]
    async fn list_records_page_returns_records() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/appTEST/tblASSETS"))
            .and(query_param("pageSize", "100"))
            .and(header("authorization", "Bearer pat-test"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "records": [{
                    "id": "recAAA",
                    "createdTime": "2024-01-01T00:00:00.000Z",
                    "fields": { "Name": "Widget" }
                }]
            })))
            .mount(&server)
            .await;

        let client = test_app(test_config(&server.uri()));
        let page = client
            .list_records_page("assets", &AirtableListParams::default())
            .await
            .unwrap();
        assert_eq!(page.records.len(), 1);
        assert_eq!(page.records[0].id, "recAAA");
    }

    #[tokio::test]
    async fn list_all_records_follows_offset() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/appTEST/tblASSETS"))
            .and(query_param("pageSize", "100"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "records": [{ "id": "rec1", "fields": {} }],
                "offset": "page2"
            })))
            .up_to_n_times(1)
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/appTEST/tblASSETS"))
            .and(query_param("offset", "page2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "records": [{ "id": "rec2", "fields": {} }]
            })))
            .mount(&server)
            .await;

        let client = test_app_with_rate_limit(
            test_config(&server.uri()),
            AirtableRateLimitHook::new(Duration::ZERO),
        );
        let records = client
            .list_all_records("assets", AirtableListParams::default())
            .await
            .unwrap();
        assert_eq!(records.len(), 2);
    }

    #[tokio::test]
    async fn update_record_patches_single_record() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/appTEST/tblASSETS/recAAA"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "id": "recAAA",
                "fields": { "Name": "Updated" }
            })))
            .mount(&server)
            .await;

        let client = test_app(test_config(&server.uri()));
        let mut fields = AirtableFields::new();
        fields.insert("Name", json!("Updated"));
        let record = client
            .update_record("assets", "recAAA", fields)
            .await
            .unwrap();
        assert_eq!(record.id, "recAAA");
    }

    #[tokio::test]
    async fn batch_update_records_chunks_requests() {
        let server = MockServer::start().await;
        Mock::given(method("PATCH"))
            .and(path("/appTEST/tblASSETS"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "records": [{ "id": "rec1", "fields": {} }, { "id": "rec2", "fields": {} }]
            })))
            .expect(2)
            .mount(&server)
            .await;

        let updates: Vec<_> = (1..=12)
            .map(|index| AirtableBatchUpdate {
                id: format!("rec{index}"),
                fields: AirtableFields::new(),
            })
            .collect();

        let client = test_app_with_rate_limit(
            test_config(&server.uri()),
            AirtableRateLimitHook::new(Duration::ZERO),
        );
        let records = client
            .batch_update_records("assets", updates)
            .await
            .unwrap();
        assert_eq!(records.len(), 4);
    }

    #[test]
    fn config_from_document_reads_tables() {
        std::env::set_var("AIRTABLE_TOKEN", "pat-test");
        let document = ConfigDocument::parse_toml(
            r#"
[airtable]
base_id = "appTEST"

[airtable.tables.assets]
table_id = "tblASSETS"
primary_key_field = "Asset ID"
"#,
        )
        .unwrap();
        let config = AirtableConfig::from_document(&document).unwrap();
        assert_eq!(config.table("assets").unwrap().table_id, "tblASSETS");
    }

    #[test]
    fn unknown_table_returns_error_code() {
        let config = AirtableConfig::builder("appTEST", "pat-test")
            .build()
            .unwrap();
        let err = config.table("missing").unwrap_err();
        assert_eq!(err.code(), Some(codes::NEST_AIRTABLE_TABLE_NOT_FOUND));
    }

    #[tokio::test]
    async fn get_base_schema_returns_tables() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/meta/bases/appTEST/tables"))
            .and(header("authorization", "Bearer pat-test"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "tables": [{
                    "id": "tblASSETS",
                    "name": "Assets",
                    "primaryFieldId": "fldKEY",
                    "fields": [{
                        "id": "fldKEY",
                        "name": "Name",
                        "type": "singleLineText"
                    }]
                }]
            })))
            .mount(&server)
            .await;

        let config = AirtableConfig::builder("appTEST", "pat-test")
            .meta_api_url(format!("{}/meta", server.uri()))
            .table("assets", "tblASSETS", None)
            .build()
            .unwrap();
        let client = test_app(config);
        let schema = client.get_base_schema().await.unwrap();
        assert_eq!(schema.tables.len(), 1);
        assert_eq!(schema.tables[0].id, "tblASSETS");
    }
}
