//! Scan result models.

use nest_media::{MediaInspection, MovieMetadata};

use crate::config::LibraryId;

/// One file discovered during a library scan.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ScannedFile {
    /// Path relative to the file service scope.
    pub relative_path: String,
    /// File size in bytes.
    pub size_bytes: u64,
    /// Last modification time as unix seconds when available.
    pub modified_secs: Option<u64>,
}

/// Status of one scanned movie candidate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ScanItemStatus {
    /// Newly discovered file.
    New,
    /// Updated since the previous scan.
    Updated,
    /// Unchanged since the previous scan.
    Unchanged,
    /// Skipped by policy or options.
    Skipped,
    /// Failed to process.
    Error,
}

/// One movie candidate discovered from the filesystem.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MovieScanCandidate {
    /// Source file metadata.
    pub file: ScannedFile,
    /// Title guessed from the filename or path.
    pub guessed_title: Option<String>,
    /// Year guessed from the filename or path.
    pub guessed_year: Option<u16>,
    /// Technical inspection results when requested.
    pub inspection: Option<MediaInspection>,
    /// Provider metadata when requested.
    pub metadata: Option<MovieMetadata>,
    /// Candidate processing status.
    pub status: ScanItemStatus,
}

/// One scan failure tied to a path.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ScanError {
    /// File or directory path.
    pub path: String,
    /// Error message.
    pub message: String,
}

/// Aggregate scan counters.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ScanStats {
    /// Total files examined.
    pub files_seen: u32,
    /// Movie candidates discovered.
    pub candidates: u32,
    /// Errors encountered.
    pub errors: u32,
}

/// Result of scanning one media library.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ScanResult {
    /// Library that was scanned.
    pub library_id: LibraryId,
    /// Scan start time as unix seconds.
    pub started_at: u64,
    /// Scan end time as unix seconds.
    pub finished_at: u64,
    /// Discovered movie candidates.
    pub candidates: Vec<MovieScanCandidate>,
    /// Non-fatal scan errors.
    pub errors: Vec<ScanError>,
    /// Aggregate counters.
    pub stats: ScanStats,
}

/// Options controlling indexer behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LibraryScanOptions {
    /// Inspect files with [`nest_media::MediaInspector`].
    pub inspect_files: bool,
    /// Fetch metadata with [`nest_media::MetadataProvider`].
    pub fetch_metadata: bool,
    /// Persist movies with [`nest_media::MediaLibraryRepository`].
    pub persist: bool,
}

impl LibraryScanOptions {
    /// Discovery-only scan.
    pub fn discover_only() -> Self {
        Self::default()
    }

    /// Full pipeline when all providers are configured.
    pub fn full() -> Self {
        Self {
            inspect_files: true,
            fetch_metadata: true,
            persist: true,
        }
    }
}
