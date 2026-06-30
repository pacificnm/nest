//! CSV read/write options.

use std::collections::HashMap;

/// Maps source CSV header names to target field names.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CsvColumnMapping {
    /// Source header (normalized) to target field name.
    pub source_to_target: HashMap<String, String>,
}

impl CsvColumnMapping {
    /// Creates an empty mapping.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts a source-to-target column mapping.
    pub fn insert(&mut self, source: impl Into<String>, target: impl Into<String>) {
        self.source_to_target
            .insert(source.into(), target.into());
    }

    /// Returns the target name for a normalized source header.
    pub fn map_header(&self, normalized_source: &str) -> String {
        self.source_to_target
            .get(normalized_source)
            .cloned()
            .unwrap_or_else(|| normalized_source.to_string())
    }
}

/// Options for CSV read and write operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvOptions {
    /// Whether the first row contains headers.
    pub has_headers: bool,
    /// Field delimiter byte.
    pub delimiter: u8,
    /// Allow variable column counts per row.
    pub flexible: bool,
    /// Trim whitespace from fields and headers.
    pub trim: bool,
    /// Lowercase headers for mapping lookup.
    pub normalize_lowercase: bool,
    /// Source-to-target column mapping.
    pub column_mapping: CsvColumnMapping,
    /// Required target column names after mapping.
    pub required_columns: Vec<String>,
    /// Maximum row issues before aborting.
    pub max_errors: Option<usize>,
    /// Continue processing after row errors.
    pub continue_on_error: bool,
    /// Create parent directories when writing.
    pub create_parent_dirs: bool,
}

impl Default for CsvOptions {
    fn default() -> Self {
        Self {
            has_headers: true,
            delimiter: b',',
            flexible: false,
            trim: true,
            normalize_lowercase: true,
            column_mapping: CsvColumnMapping::new(),
            required_columns: Vec::new(),
            max_errors: None,
            continue_on_error: false,
            create_parent_dirs: true,
        }
    }
}

impl CsvOptions {
    /// Maps a source column header to a target field name.
    pub fn map_column(mut self, source: impl Into<String>, target: impl Into<String>) -> Self {
        self.column_mapping.insert(source, target);
        self
    }

    /// Sets required target columns.
    pub fn require_columns<I, S>(mut self, columns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.required_columns = columns.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the delimiter.
    pub fn delimiter(mut self, delimiter: u8) -> Self {
        self.delimiter = delimiter;
        self
    }

    /// Sets whether to trim fields.
    pub fn trim(mut self, trim: bool) -> Self {
        self.trim = trim;
        self
    }

    /// Sets whether to continue on row errors.
    pub fn continue_on_error(mut self, continue_on_error: bool) -> Self {
        self.continue_on_error = continue_on_error;
        self
    }

    /// Sets the maximum number of row issues before aborting.
    pub fn max_errors(mut self, max_errors: usize) -> Self {
        self.max_errors = Some(max_errors);
        self
    }
}

/// Normalizes a header for mapping lookup.
pub fn normalize_header(name: &str, trim: bool, lowercase: bool) -> String {
    let mut value = if trim { name.trim() } else { name }.to_string();
    if lowercase {
        value = value.to_lowercase();
    }
    value
}
