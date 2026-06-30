//! CSV reading and parsing.

use std::collections::BTreeMap;
use std::io::Cursor;
use std::path::Path;

use nest_file::FileService;
use serde::de::DeserializeOwned;
use tracing::{info, warn};

use crate::codes::{
    NEST_CSV_DESERIALIZE_FAILED, NEST_CSV_PARSE_FAILED, NEST_CSV_REQUIRED_COLUMN_MISSING,
    NEST_CSV_ROW_LIMIT_EXCEEDED, NEST_CSV_VALIDATION_FAILED,
};
use crate::error::{CsvError, CsvResult};
use crate::options::{normalize_header, CsvOptions};
use crate::record::CsvRecord;
use crate::report::{CsvReadReport, CsvRowIssue};
use crate::validator::CsvRowValidator;

struct PreparedHeaders {
    target_headers: Vec<String>,
}

pub(crate) fn read_records(
    files: &FileService,
    path: &Path,
    options: &CsvOptions,
) -> CsvResult<Vec<CsvRecord>> {
    let report = read_records_report(files, path, options)?;
    strict_from_report(report, path)
}

pub(crate) fn read_records_report(
    files: &FileService,
    path: &Path,
    options: &CsvOptions,
) -> CsvResult<CsvReadReport<CsvRecord>> {
    let content = files
        .read_text(path)
        .map_err(|error| CsvError::parse(error.to_string()).with_path(path))?;

    let mut reader = build_reader(content.as_bytes(), options);
    let prepared = prepare_headers(&mut reader, options)?;
    let mut report = CsvReadReport::new();

    for (index, result) in reader.records().enumerate() {
        let row_number = index + 1;
        report.total_rows += 1;

        let record = match result {
            Ok(record) => record,
            Err(error) => {
                add_issue(
                    &mut report,
                    row_number,
                    NEST_CSV_PARSE_FAILED,
                    error.to_string(),
                    None,
                );
                if should_stop(options, &report.issues) {
                    break;
                }
                continue;
            }
        };

        match record_to_csv_record(&record, &prepared, options) {
            Ok(row) => {
                report.rows.push(row);
                report.valid_rows += 1;
            }
            Err(issue) => {
                add_issue_struct(&mut report, issue);
                if should_stop(options, &report.issues) {
                    break;
                }
            }
        }
    }

    log_read(path, &report);
    Ok(report)
}

pub(crate) fn read_typed_report<T, V>(
    files: &FileService,
    path: &Path,
    options: &CsvOptions,
    validator: Option<&V>,
) -> CsvResult<CsvReadReport<T>>
where
    T: DeserializeOwned,
    V: CsvRowValidator<T>,
{
    read_typed_report_inner(files, path, options, validator, |_, _| Vec::new())
}

#[cfg(feature = "validate")]
pub(crate) fn read_typed_validated_report<T>(
    files: &FileService,
    path: &Path,
    options: &CsvOptions,
) -> CsvResult<CsvReadReport<T>>
where
    T: DeserializeOwned + nest_validation::Validate,
{
    read_typed_report_inner(
        files,
        path,
        options,
        None::<&NoopRowValidator>,
        |row, row_number| crate::validator::validate_row(row, row_number),
    )
}

#[allow(dead_code)]
struct NoopRowValidator;

impl<T> CsvRowValidator<T> for NoopRowValidator {
    fn validate_row(&self, _row: &T, _row_number: usize) -> Vec<CsvRowIssue> {
        Vec::new()
    }
}

fn read_typed_report_inner<T, V, F>(
    files: &FileService,
    path: &Path,
    options: &CsvOptions,
    validator: Option<&V>,
    extra_validate: F,
) -> CsvResult<CsvReadReport<T>>
where
    T: DeserializeOwned,
    V: CsvRowValidator<T>,
    F: Fn(&T, usize) -> Vec<CsvRowIssue>,
{
    let content = files
        .read_text(path)
        .map_err(|error| CsvError::parse(error.to_string()).with_path(path))?;

    let mut reader = build_reader(content.as_bytes(), options);
    let prepared = prepare_headers(&mut reader, options)?;
    let mut report = CsvReadReport::new();

    for (index, result) in reader.records().enumerate() {
        let row_number = index + 1;
        report.total_rows += 1;

        let record = match result {
            Ok(record) => record,
            Err(error) => {
                add_issue(
                    &mut report,
                    row_number,
                    NEST_CSV_PARSE_FAILED,
                    error.to_string(),
                    None,
                );
                if should_stop(options, &report.issues) {
                    break;
                }
                continue;
            }
        };

        let mapped = match record_to_csv_record(&record, &prepared, options) {
            Ok(row) => row,
            Err(issue) => {
                add_issue_struct(&mut report, issue);
                if should_stop(options, &report.issues) {
                    break;
                }
                continue;
            }
        };

        match deserialize_record::<T>(&mapped, &prepared.target_headers) {
            Ok(row) => {
                let mut row_issues = Vec::new();
                if let Some(custom) = validator {
                    row_issues.extend(custom.validate_row(&row, row_number));
                }
                row_issues.extend(extra_validate(&row, row_number));

                if row_issues.is_empty() {
                    report.rows.push(row);
                    report.valid_rows += 1;
                } else {
                    report.skipped_rows += 1;
                    report.issues.extend(row_issues);
                    if should_stop(options, &report.issues) {
                        break;
                    }
                }
            }
            Err(message) => {
                add_issue(
                    &mut report,
                    row_number,
                    NEST_CSV_DESERIALIZE_FAILED,
                    message,
                    None,
                );
                if should_stop(options, &report.issues) {
                    break;
                }
            }
        }
    }

    log_read(path, &report);
    Ok(report)
}

fn build_reader<'a>(content: &'a [u8], options: &CsvOptions) -> csv::Reader<Cursor<&'a [u8]>> {
    let mut builder = csv::ReaderBuilder::new();
    builder
        .has_headers(options.has_headers)
        .delimiter(options.delimiter)
        .flexible(options.flexible)
        .from_reader(Cursor::new(content))
}

fn prepare_headers(
    reader: &mut csv::Reader<Cursor<&[u8]>>,
    options: &CsvOptions,
) -> CsvResult<PreparedHeaders> {
    if !options.has_headers {
        return Ok(PreparedHeaders {
            target_headers: Vec::new(),
        });
    }

    let headers = reader
        .headers()
        .map_err(|error| CsvError::parse(error.to_string()))?
        .clone();

    let mut target_headers = Vec::with_capacity(headers.len());
    for header in headers.iter() {
        let normalized = normalize_header(header, options.trim, options.normalize_lowercase);
        target_headers.push(options.column_mapping.map_header(&normalized));
    }

    for required in &options.required_columns {
        if !target_headers.iter().any(|header| header == required) {
            return Err(CsvError::required_column(format!(
                "required column missing: {required}"
            )));
        }
    }

    Ok(PreparedHeaders { target_headers })
}

fn record_to_csv_record(
    record: &csv::StringRecord,
    prepared: &PreparedHeaders,
    options: &CsvOptions,
) -> Result<CsvRecord, CsvRowIssue> {
    let mut fields = BTreeMap::new();

    if options.has_headers {
        if record.len() != prepared.target_headers.len() && !options.flexible {
            return Err(CsvRowIssue::new(
                0,
                NEST_CSV_PARSE_FAILED,
                format!(
                    "expected {} columns, found {}",
                    prepared.target_headers.len(),
                    record.len()
                ),
            ));
        }

        for (index, header) in prepared.target_headers.iter().enumerate() {
            let value = record.get(index).unwrap_or("");
            let value = if options.trim { value.trim() } else { value };
            fields.insert(header.clone(), value.to_string());
        }
    } else {
        for (index, value) in record.iter().enumerate() {
            let value = if options.trim { value.trim() } else { value };
            fields.insert(format!("column_{index}"), value.to_string());
        }
    }

    Ok(CsvRecord { fields })
}

fn deserialize_record<T: DeserializeOwned>(
    record: &CsvRecord,
    headers: &[String],
) -> Result<T, String> {
    let mut writer = csv::Writer::from_writer(Vec::new());
    writer
        .write_record(headers.iter().map(String::as_str))
        .map_err(|error| error.to_string())?;

    let values: Vec<&str> = headers
        .iter()
        .map(|header| record.fields.get(header).map(String::as_str).unwrap_or(""))
        .collect();
    writer
        .write_record(&values)
        .map_err(|error| error.to_string())?;

    let data = writer.into_inner().map_err(|error| error.to_string())?;
    let mut reader = csv::Reader::from_reader(Cursor::new(data));
    reader
        .deserialize()
        .next()
        .ok_or_else(|| "missing CSV row".to_string())?
        .map_err(|error| error.to_string())
}

fn add_issue<T>(
    report: &mut CsvReadReport<T>,
    row_number: usize,
    code: &str,
    message: String,
    column: Option<String>,
) {
    report.skipped_rows += 1;
    let mut issue = CsvRowIssue::new(row_number, code, message);
    if let Some(column) = column {
        issue = issue.with_column(column);
    }
    report.issues.push(issue);
}

fn add_issue_struct<T>(report: &mut CsvReadReport<T>, issue: CsvRowIssue) {
    report.skipped_rows += 1;
    report.issues.push(issue);
}

fn should_stop(options: &CsvOptions, issues: &[CsvRowIssue]) -> bool {
    if let Some(max) = options.max_errors {
        if issues.len() >= max {
            return true;
        }
    }
    !options.continue_on_error
}

pub(crate) fn strict_from_report<T>(report: CsvReadReport<T>, path: &Path) -> CsvResult<Vec<T>> {
    if report.issues.is_empty() {
        return Ok(report.rows);
    }

    let first = &report.issues[0];
    let error = match first.code.as_str() {
        NEST_CSV_ROW_LIMIT_EXCEEDED => CsvError::row_limit(first.message.clone()),
        NEST_CSV_PARSE_FAILED => CsvError::parse(first.message.clone()),
        NEST_CSV_REQUIRED_COLUMN_MISSING => CsvError::required_column(first.message.clone()),
        NEST_CSV_VALIDATION_FAILED => CsvError::validation(first.message.clone()),
        _ => CsvError::deserialize(first.message.clone()),
    };
    Err(error.with_path(path))
}

fn log_read<T>(path: &Path, report: &CsvReadReport<T>) {
    if report.issues.is_empty() {
        info!(
            file.path = %path.display(),
            csv.total_rows = report.total_rows,
            csv.valid_rows = report.valid_rows,
            "csv read"
        );
    } else {
        warn!(
            file.path = %path.display(),
            csv.total_rows = report.total_rows,
            csv.valid_rows = report.valid_rows,
            csv.skipped_rows = report.skipped_rows,
            csv.issues = report.issues.len(),
            "csv read with issues"
        );
    }
}
