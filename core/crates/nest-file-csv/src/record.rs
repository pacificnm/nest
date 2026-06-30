//! CSV record representation.

use std::collections::BTreeMap;

/// A single CSV row as mapped field name → value.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CsvRecord {
    /// Target field names to cell values.
    pub fields: BTreeMap<String, String>,
}

impl CsvRecord {
    /// Creates an empty record.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a field value by name.
    pub fn get(&self, field: &str) -> Option<&str> {
        self.fields.get(field).map(String::as_str)
    }
}
