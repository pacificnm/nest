//! Row validation hooks.

use crate::report::CsvRowIssue;

/// Validates a deserialized CSV row.
pub trait CsvRowValidator<T>: Send + Sync {
    /// Returns row issues for the given row.
    fn validate_row(&self, row: &T, row_number: usize) -> Vec<CsvRowIssue>;
}

#[cfg(feature = "validate")]
pub(crate) fn validate_row<T: nest_validation::Validate>(
    row: &T,
    row_number: usize,
) -> Vec<CsvRowIssue> {
    match nest_validation::validate(row) {
        Ok(()) => Vec::new(),
        Err(error) => error
            .issues()
            .iter()
            .filter(|issue| issue.is_blocking())
            .map(|issue| {
                let mut csv_issue = CsvRowIssue::new(
                    row_number,
                    issue.code.clone(),
                    issue.message.clone(),
                );
                if let Some(field) = &issue.field {
                    csv_issue = csv_issue.with_column(field.as_str().to_string());
                }
                csv_issue
            })
            .collect(),
    }
}
