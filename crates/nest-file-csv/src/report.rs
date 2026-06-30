//! CSV import/export reports.

/// A row-level CSV issue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvRowIssue {
    /// 1-based data row number (excluding header).
    pub row_number: usize,
    /// Column name when applicable.
    pub column: Option<String>,
    /// Stable issue code.
    pub code: String,
    /// Human-readable message.
    pub message: String,
}

impl CsvRowIssue {
    /// Creates a new row issue.
    pub fn new(
        row_number: usize,
        code: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            row_number,
            column: None,
            code: code.into(),
            message: message.into(),
        }
    }

    /// Sets the column name.
    pub fn with_column(mut self, column: impl Into<String>) -> Self {
        self.column = Some(column.into());
        self
    }
}

/// Result of reading typed CSV rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvReadReport<T> {
    /// Successfully parsed rows.
    pub rows: Vec<T>,
    /// Total data rows encountered.
    pub total_rows: usize,
    /// Successfully parsed row count.
    pub valid_rows: usize,
    /// Skipped row count.
    pub skipped_rows: usize,
    /// Row-level issues.
    pub issues: Vec<CsvRowIssue>,
}

impl<T> CsvReadReport<T> {
    /// Creates an empty report.
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            total_rows: 0,
            valid_rows: 0,
            skipped_rows: 0,
            issues: Vec::new(),
        }
    }
}

impl<T> Default for CsvReadReport<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of writing CSV rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvWriteReport {
    /// Rows written.
    pub rows_written: usize,
    /// Row-level issues.
    pub issues: Vec<CsvRowIssue>,
}

impl CsvWriteReport {
    /// Creates an empty write report.
    pub fn new() -> Self {
        Self {
            rows_written: 0,
            issues: Vec::new(),
        }
    }
}

impl Default for CsvWriteReport {
    fn default() -> Self {
        Self::new()
    }
}
