//! CSV writing.

use std::io::Cursor;
use std::path::Path;

use nest_file::{FileService, WriteOptions};
use serde::Serialize;
use tracing::{info, warn};

use crate::codes::NEST_CSV_WRITE_FAILED;
use crate::error::{CsvError, CsvResult};
use crate::options::CsvOptions;
use crate::record::CsvRecord;
use crate::report::{CsvRowIssue, CsvWriteReport};

pub(crate) fn write_records(
    files: &FileService,
    path: &Path,
    records: &[CsvRecord],
    options: &CsvOptions,
) -> CsvResult<CsvWriteReport> {
    let mut writer = build_writer(options);
    let mut report = CsvWriteReport::new();

    if options.has_headers {
        if let Some(first) = records.first() {
            let headers: Vec<&str> = first.fields.keys().map(String::as_str).collect();
            if let Err(error) = writer.write_record(&headers) {
                return Err(CsvError::write(error.to_string()).with_path(path));
            }
        }
    }

    for (index, record) in records.iter().enumerate() {
        let row_number = index + 1;
        let values: Vec<&str> = if options.has_headers {
            if let Some(first) = records.first() {
                first
                    .fields
                    .keys()
                    .map(|key| record.fields.get(key).map(String::as_str).unwrap_or(""))
                    .collect()
            } else {
                record.fields.values().map(String::as_str).collect()
            }
        } else {
            record.fields.values().map(String::as_str).collect()
        };

        if let Err(error) = writer.write_record(&values) {
            report.issues.push(
                CsvRowIssue::new(row_number, NEST_CSV_WRITE_FAILED, error.to_string()),
            );
            continue;
        }
        report.rows_written += 1;
    }

    flush_writer(files, path, &mut writer, options, &report)
}

pub(crate) fn write_typed<T: Serialize>(
    files: &FileService,
    path: &Path,
    rows: &[T],
    options: &CsvOptions,
) -> CsvResult<CsvWriteReport> {
    let mut writer = build_writer(options);
    let mut report = CsvWriteReport::new();

    for (index, row) in rows.iter().enumerate() {
        let row_number = index + 1;
        if let Err(error) = writer.serialize(row) {
            report.issues.push(
                CsvRowIssue::new(row_number, NEST_CSV_WRITE_FAILED, error.to_string()),
            );
            continue;
        }
        report.rows_written += 1;
    }

    flush_writer(files, path, &mut writer, options, &report)
}

fn build_writer(options: &CsvOptions) -> csv::Writer<Cursor<Vec<u8>>> {
    let mut builder = csv::WriterBuilder::new();
    builder
        .has_headers(options.has_headers)
        .delimiter(options.delimiter)
        .flexible(options.flexible)
        .from_writer(Cursor::new(Vec::new()))
}

fn flush_writer(
    files: &FileService,
    path: &Path,
    writer: &mut csv::Writer<Cursor<Vec<u8>>>,
    options: &CsvOptions,
    report: &CsvWriteReport,
) -> CsvResult<CsvWriteReport> {
    writer
        .flush()
        .map_err(|error| CsvError::write(error.to_string()).with_path(path))?;

    let buffer = writer
        .get_ref()
        .get_ref()
        .clone();
    let content = String::from_utf8(buffer)
        .map_err(|error| CsvError::write(error.to_string()).with_path(path))?;

    let mut write_options = WriteOptions::from_config(files.config());
    if options.create_parent_dirs {
        write_options = write_options.create_parents();
    }

    files
        .write_bytes_with_options(path, content.as_bytes(), write_options)
        .map_err(|error| CsvError::write(error.to_string()).with_path(path))?;

    if report.issues.is_empty() {
        info!(
            file.path = %path.display(),
            csv.rows_written = report.rows_written,
            "csv write"
        );
    } else {
        warn!(
            file.path = %path.display(),
            csv.rows_written = report.rows_written,
            csv.issues = report.issues.len(),
            "csv write with issues"
        );
    }

    Ok(report.clone())
}
