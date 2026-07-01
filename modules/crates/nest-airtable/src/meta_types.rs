//! Airtable Meta API response types.

use serde::Deserialize;

/// Base schema returned by `GET /meta/bases/{baseId}/tables`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AirtableBaseSchema {
    /// Tables in the base.
    pub tables: Vec<AirtableTableSchema>,
}

/// Table metadata from the Meta API.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AirtableTableSchema {
    /// Table id (`tbl…`).
    pub id: String,
    /// Display name in Airtable.
    pub name: String,
    /// Primary field id (`fld…`).
    #[serde(rename = "primaryFieldId")]
    pub primary_field_id: String,
    /// Field definitions for this table.
    pub fields: Vec<AirtableFieldSchema>,
}

/// Field metadata from the Meta API.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct AirtableFieldSchema {
    /// Field id (`fld…`).
    pub id: String,
    /// Display name in Airtable.
    pub name: String,
    /// Airtable field type (e.g. `singleLineText`, `formula`).
    #[serde(rename = "type")]
    pub field_type: String,
}

/// Returns true when an Airtable field type is computed or read-only.
pub fn is_computed_field_type(field_type: &str) -> bool {
    matches!(
        field_type,
        "autoNumber"
            | "button"
            | "count"
            | "createdBy"
            | "createdTime"
            | "externalSyncSource"
            | "formula"
            | "lastModifiedBy"
            | "lastModifiedTime"
            | "lookup"
            | "multipleLookupValues"
            | "rollup"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_computed_field_types() {
        assert!(is_computed_field_type("formula"));
        assert!(is_computed_field_type("lookup"));
        assert!(!is_computed_field_type("singleLineText"));
        assert!(!is_computed_field_type("number"));
    }

    #[test]
    fn deserializes_base_schema() {
        let schema: AirtableBaseSchema = serde_json::from_str(
            r#"{
            "tables": [{
                "id": "tblTEST",
                "name": "Assets",
                "primaryFieldId": "fldKEY",
                "fields": [{
                    "id": "fldKEY",
                    "name": "Name",
                    "type": "singleLineText"
                }]
            }]
        }"#,
        )
        .unwrap();
        assert_eq!(schema.tables.len(), 1);
        assert_eq!(schema.tables[0].fields[0].field_type, "singleLineText");
    }
}
