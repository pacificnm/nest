//! Airtable record and request/response types.

use std::collections::HashMap;

use nest_task::CancelToken;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Default page size for list requests (Airtable maximum is 100).
pub const DEFAULT_PAGE_SIZE: u32 = 100;

/// Maximum records per batch update request.
pub const MAX_BATCH_UPDATE_SIZE: usize = 10;

/// JSON field map for an Airtable record.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AirtableFields(pub HashMap<String, Value>);

impl AirtableFields {
    /// Creates empty fields.
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Inserts a field value.
    pub fn insert(&mut self, name: impl Into<String>, value: Value) {
        self.0.insert(name.into(), value);
    }
}

impl From<HashMap<String, Value>> for AirtableFields {
    fn from(value: HashMap<String, Value>) -> Self {
        Self(value)
    }
}

/// One Airtable record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AirtableRecord {
    /// Record id (`rec…`).
    pub id: String,
    /// Creation timestamp from Airtable.
    #[serde(rename = "createdTime", default)]
    pub created_time: Option<String>,
    /// Field values.
    pub fields: AirtableFields,
}

/// Parameters for listing records.
#[derive(Debug, Clone, Default)]
pub struct AirtableListParams {
    /// Page size (defaults to 100).
    pub page_size: Option<u32>,
    /// Optional Airtable view name.
    pub view: Option<String>,
    /// Optional `filterByFormula` expression.
    pub filter_by_formula: Option<String>,
    /// Optional field names to return.
    pub fields: Option<Vec<String>>,
    /// Optional cooperative cancellation token.
    pub cancel: Option<CancelToken>,
}

impl AirtableListParams {
    /// Returns the effective page size.
    pub fn effective_page_size(&self) -> u32 {
        self.page_size.unwrap_or(DEFAULT_PAGE_SIZE).min(100)
    }
}

/// One page of list results.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AirtableListPage {
    /// Records in this page.
    pub records: Vec<AirtableRecord>,
    /// Offset token for the next page, if any.
    #[serde(default)]
    pub offset: Option<String>,
}

/// Update payload for a single record in a batch request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AirtableBatchUpdate {
    /// Record id to update.
    pub id: String,
    /// Field values to patch.
    pub fields: AirtableFields,
}

/// Internal API response for batch update.
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct BatchUpdateResponse {
    pub records: Vec<AirtableRecord>,
}
