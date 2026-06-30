//! CSV import/export service.

use std::path::Path;

use nest_error::NestResult;
use nest_file::FileService;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::options::CsvOptions;
use crate::reader::{
    read_records, read_records_report, read_typed_report, strict_from_report,
};
#[cfg(feature = "validate")]
use crate::reader::read_typed_validated_report;
use crate::record::CsvRecord;
use crate::report::{CsvReadReport, CsvWriteReport};
use crate::validator::CsvRowValidator;
use crate::writer::{write_records, write_typed};

/// CSV import and export using [`FileService`] for all file I/O.
#[derive(Clone)]
pub struct CsvService {
    files: FileService,
}

impl CsvService {
    /// Creates a CSV service backed by the given file service.
    pub fn new(files: FileService) -> Self {
        Self { files }
    }

    /// Returns the underlying file service.
    pub fn files(&self) -> &FileService {
        &self.files
    }

    /// Reads CSV rows as string maps.
    pub fn read_records(
        &self,
        path: impl AsRef<Path>,
        options: &CsvOptions,
    ) -> NestResult<Vec<CsvRecord>> {
        read_records(&self.files, path.as_ref(), options).map_err(Into::into)
    }

    /// Reads CSV rows as string maps with a full report.
    pub fn read_records_report(
        &self,
        path: impl AsRef<Path>,
        options: &CsvOptions,
    ) -> NestResult<CsvReadReport<CsvRecord>> {
        read_records_report(&self.files, path.as_ref(), options).map_err(Into::into)
    }

    /// Reads and deserializes CSV rows (strict unless `continue_on_error` is set).
    pub fn read_typed<T: DeserializeOwned>(
        &self,
        path: impl AsRef<Path>,
        options: &CsvOptions,
    ) -> NestResult<Vec<T>> {
        let path = path.as_ref();
        let report = self.read_typed_report::<T>(path, options)?;
        if options.continue_on_error {
            Ok(report.rows)
        } else {
            strict_from_report(report, path).map_err(Into::into)
        }
    }

    /// Reads and deserializes CSV rows, collecting issues in a report.
    pub fn read_typed_report<T: DeserializeOwned>(
        &self,
        path: impl AsRef<Path>,
        options: &CsvOptions,
    ) -> NestResult<CsvReadReport<T>> {
        read_typed_report::<T, NoopValidator>(
            &self.files,
            path.as_ref(),
            options,
            None,
        )
        .map_err(Into::into)
    }

    /// Reads typed rows with a custom row validator.
    pub fn read_typed_report_with_validator<T, V>(
        &self,
        path: impl AsRef<Path>,
        options: &CsvOptions,
        validator: &V,
    ) -> NestResult<CsvReadReport<T>>
    where
        T: DeserializeOwned,
        V: CsvRowValidator<T>,
    {
        read_typed_report(&self.files, path.as_ref(), options, Some(validator))
            .map_err(Into::into)
    }

    /// Reads typed rows and runs [`nest_validation::Validate`] when the `validate` feature is enabled.
    #[cfg(feature = "validate")]
    pub fn read_typed_with_validation<T>(
        &self,
        path: impl AsRef<Path>,
        options: &CsvOptions,
    ) -> NestResult<CsvReadReport<T>>
    where
        T: DeserializeOwned + nest_validation::Validate,
    {
        read_typed_validated_report(&self.files, path.as_ref(), options)
        .map_err(Into::into)
    }

    /// Writes CSV records to a file.
    pub fn write_records(
        &self,
        path: impl AsRef<Path>,
        records: &[CsvRecord],
        options: &CsvOptions,
    ) -> NestResult<CsvWriteReport> {
        write_records(&self.files, path.as_ref(), records, options).map_err(Into::into)
    }

    /// Writes typed rows to a CSV file.
    pub fn write_typed<T: Serialize>(
        &self,
        path: impl AsRef<Path>,
        rows: &[T],
        options: &CsvOptions,
    ) -> NestResult<CsvWriteReport> {
        write_typed(&self.files, path.as_ref(), rows, options).map_err(Into::into)
    }
}

struct NoopValidator;

impl<T> CsvRowValidator<T> for NoopValidator {
    fn validate_row(&self, _row: &T, _row_number: usize) -> Vec<crate::report::CsvRowIssue> {
        Vec::new()
    }
}
