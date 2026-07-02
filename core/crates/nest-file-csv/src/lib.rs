//! CSV import and export for the Nest framework.
//!
//! nest-file-csv parses, maps, validates, and reports CSV data using
//! [`nest_file::FileService`] for all file I/O.

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

pub mod codes;
mod error;
mod module;
mod options;
pub mod prelude;
mod reader;
mod record;
mod report;
mod service;
mod validator;
mod writer;

pub use error::{CsvError, CsvErrorKind, CsvResult};
pub use module::{CsvModule, CSV_MODULE_ID};
pub use options::{normalize_header, CsvColumnMapping, CsvOptions};
pub use record::CsvRecord;
pub use report::{CsvReadReport, CsvRowIssue, CsvWriteReport};
pub use service::CsvService;
pub use validator::CsvRowValidator;

pub use nest_core::{Module, ModuleId};
pub use nest_error::{NestError, NestResult};
pub use nest_file::FileService;

#[cfg(test)]
mod tests {
    use nest_core::AppBuilder;
    use nest_error::{codes, NestErrorKind};
    use nest_file::{FileModule, FileServiceConfig};
    use serde::{Deserialize, Serialize};
    use tempfile::tempdir;

    use super::*;
    use crate::module::CsvModule;

    #[derive(Debug, Deserialize, Serialize, PartialEq)]
    struct AirtableImportRow {
        customer_id: String,
        email: String,
        city: Option<String>,
    }

    fn csv_service(root: &std::path::Path) -> CsvService {
        let built = AppBuilder::new()
            .module(FileModule::with_config(
                FileServiceConfig::scoped(root).allow_create_dirs(true),
            ))
            .module(CsvModule)
            .build()
            .unwrap();
        built.context.service::<CsvService>().unwrap().clone()
    }

    #[test]
    fn read_records_basic() {
        let dir = tempdir().unwrap();
        let csv = csv_service(dir.path());
        csv.files()
            .write_text(
                "customers.csv",
                "customer_id,email\nCUST-1,alice@example.com\n",
            )
            .unwrap();

        let options = CsvOptions::default();
        let rows = csv.read_records("customers.csv", &options).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].get("customer_id"), Some("CUST-1"));
    }

    #[test]
    fn column_mapping_deserializes_typed_rows() {
        let dir = tempdir().unwrap();
        let csv = csv_service(dir.path());
        csv.files()
            .write_text(
                "imports/customers.csv",
                "cust_id,eml,city\nCUST-1,alice@example.com,Portland\n",
            )
            .unwrap();

        let options = CsvOptions::default()
            .map_column("cust_id", "customer_id")
            .map_column("eml", "email");

        let rows: Vec<AirtableImportRow> =
            csv.read_typed("imports/customers.csv", &options).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].customer_id, "CUST-1");
        assert_eq!(rows[0].email, "alice@example.com");
        assert_eq!(rows[0].city.as_deref(), Some("Portland"));
    }

    #[test]
    fn required_column_missing_fails() {
        let dir = tempdir().unwrap();
        let csv = csv_service(dir.path());
        csv.files()
            .write_text("bad.csv", "customer_id\nCUST-1\n")
            .unwrap();

        let options = CsvOptions::default().require_columns(["customer_id", "email"]);
        let err = csv
            .read_typed::<AirtableImportRow>("bad.csv", &options)
            .unwrap_err();
        assert_eq!(err.code(), Some(codes::NEST_CSV_REQUIRED_COLUMN_MISSING));
    }

    #[test]
    fn continue_on_error_collects_issues() {
        let dir = tempdir().unwrap();
        let csv = csv_service(dir.path());
        csv.files()
            .write_text(
                "mixed.csv",
                "customer_id,email,city\nCUST-1,alice@example.com,Portland\nCUST-2,bob@example.com\n",
            )
            .unwrap();

        let options = CsvOptions::default().continue_on_error(true);
        let report = csv
            .read_typed_report::<AirtableImportRow>("mixed.csv", &options)
            .unwrap();
        assert_eq!(report.valid_rows, 1);
        assert_eq!(report.skipped_rows, 1);
        assert!(!report.issues.is_empty());
    }

    #[test]
    fn strict_read_typed_fails_on_bad_row() {
        let dir = tempdir().unwrap();
        let csv = csv_service(dir.path());
        csv.files()
            .write_text(
                "mixed.csv",
                "customer_id,email,city\nCUST-1,alice@example.com,Portland\nCUST-2,bob@example.com\n",
            )
            .unwrap();

        let options = CsvOptions::default();
        assert!(csv
            .read_typed::<AirtableImportRow>("mixed.csv", &options)
            .is_err());
    }

    #[test]
    fn header_trim_and_normalize_maps_columns() {
        let dir = tempdir().unwrap();
        let csv = csv_service(dir.path());
        csv.files()
            .write_text(
                "trimmed.csv",
                " Customer ID , Email \nCUST-3,carol@example.com\n",
            )
            .unwrap();

        let options = CsvOptions::default()
            .map_column("customer id", "customer_id")
            .map_column("email", "email");

        let rows: Vec<AirtableImportRow> = csv.read_typed("trimmed.csv", &options).unwrap();
        assert_eq!(rows[0].customer_id, "CUST-3");
        assert_eq!(rows[0].email, "carol@example.com");
    }

    #[test]
    fn max_errors_stops_after_threshold() {
        let dir = tempdir().unwrap();
        let csv = csv_service(dir.path());
        csv.files()
            .write_text(
                "many-bad.csv",
                "customer_id,email,city\nCUST-1,alice@example.com,Portland\nCUST-2,bob@example.com\nCUST-3,carol@example.com\n",
            )
            .unwrap();

        let options = CsvOptions::default().continue_on_error(true).max_errors(2);
        let report = csv
            .read_typed_report::<AirtableImportRow>("many-bad.csv", &options)
            .unwrap();
        assert_eq!(report.issues.len(), 2);
        assert_eq!(report.valid_rows, 1);
    }

    #[test]
    fn deserialize_error_uses_nest_csv_code() {
        #[derive(Debug, Deserialize)]
        struct CountRow {
            #[allow(dead_code)]
            customer_id: String,
            count: u32,
        }

        let dir = tempdir().unwrap();
        let csv = csv_service(dir.path());
        csv.files()
            .write_text("bad.csv", "customer_id,count\nCUST-1,not-a-number\n")
            .unwrap();

        let err = csv
            .read_typed::<CountRow>("bad.csv", &CsvOptions::default())
            .unwrap_err();
        assert_eq!(err.code(), Some(codes::NEST_CSV_DESERIALIZE_FAILED));
    }

    #[test]
    fn write_typed_round_trip() {
        let dir = tempdir().unwrap();
        let csv = csv_service(dir.path());
        let rows = vec![AirtableImportRow {
            customer_id: "CUST-2".to_string(),
            email: "bob@example.com".to_string(),
            city: Some("Salem".to_string()),
        }];

        let options = CsvOptions::default();
        csv.write_typed("exports/customers.csv", &rows, &options)
            .unwrap();

        let loaded: Vec<AirtableImportRow> =
            csv.read_typed("exports/customers.csv", &options).unwrap();
        assert_eq!(loaded, rows);
    }

    #[test]
    fn csv_error_converts_to_nest_error() {
        let error = CsvError::required_column("missing email");
        let nest_error: NestError = error.into();
        assert_eq!(nest_error.kind(), NestErrorKind::Validation);
        assert_eq!(
            nest_error.code(),
            Some(codes::NEST_CSV_REQUIRED_COLUMN_MISSING)
        );
    }

    #[cfg(feature = "validate")]
    #[test]
    fn validate_feature_rejects_invalid_rows() {
        use nest_validation::{Validate, ValidationContext, ValidationIssue, ValidationResult};

        #[derive(Debug, Deserialize)]
        struct ValidatedRow {
            email: String,
        }

        impl Validate for ValidatedRow {
            fn validate(&self, _ctx: &ValidationContext) -> ValidationResult {
                if self.email.contains('@') {
                    Ok(())
                } else {
                    nest_validation::ValidationError::from_issues(vec![
                        ValidationIssue::field_error("email", "validation.email", "invalid email"),
                    ])
                }
            }
        }

        let dir = tempdir().unwrap();
        let csv = csv_service(dir.path());
        csv.files()
            .write_text("validated.csv", "email\nbad\nok@example.com\n")
            .unwrap();

        let options = CsvOptions::default().continue_on_error(true);
        let report = csv
            .read_typed_with_validation::<ValidatedRow>("validated.csv", &options)
            .unwrap();
        assert_eq!(report.valid_rows, 1);
        assert_eq!(report.skipped_rows, 1);
    }
}
