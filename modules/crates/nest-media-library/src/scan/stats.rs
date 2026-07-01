//! Scan statistics helpers.

use super::models::{ScanError, ScanResult, ScanStats};

/// Initializes scan stats for a new result.
pub(crate) fn initial_stats() -> ScanStats {
    ScanStats::default()
}

/// Records one examined file.
pub(crate) fn record_file_seen(stats: &mut ScanStats) {
    stats.files_seen += 1;
}

/// Records one discovered candidate.
pub(crate) fn record_candidate(stats: &mut ScanStats) {
    stats.candidates += 1;
}

/// Records one scan error on the result.
pub(crate) fn record_error(result: &mut ScanResult, path: impl Into<String>, message: impl Into<String>) {
    result.errors.push(ScanError {
        path: path.into(),
        message: message.into(),
    });
    result.stats.errors += 1;
}
