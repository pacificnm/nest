//! Media library scanning and indexing for the Nest framework.
//!
//! `nest-media-library` discovers video files via [`nest_file::FileService`],
//! applies filename heuristics, and optionally enriches candidates through
//! injected [`nest_media`] provider traits.
//!
//! # Quick start
//!
//! ```no_run
//! use nest_core::AppBuilder;
//! use nest_file::FileModule;
//! use nest_media_library::{MediaLibraryConfig, MediaLibraryModule, LibraryScanner};
//!
//! let mut built = AppBuilder::new()
//!     .module(FileModule::scoped("./media"))
//!     .module(MediaLibraryModule::new())
//!     .build()
//!     .unwrap();
//! built.startup().unwrap();
//!
//! let scanner = built.context.service::<LibraryScanner>().unwrap();
//! let config = MediaLibraryConfig::new("main", ["Movies"]);
//! let result = scanner.discover(&config).unwrap();
//! ```

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

mod codes;
mod config;
mod error;
mod indexer;
mod module;
mod parse;
mod scan;
mod task;

pub mod prelude;

pub use config::{LibraryId, MediaLibraryConfig};
pub use error::{LibraryError, LibraryErrorKind, LibraryResult};
pub use indexer::LibraryIndexer;
pub use module::{MediaLibraryModule, MEDIA_LIBRARY_MODULE_ID};
pub use scan::{
    LibraryScanOptions, LibraryScanner, MovieScanCandidate, ScanError, ScanItemStatus,
    ScanResult, ScanStats, ScannedFile,
};
pub use task::LibraryScanTask;

pub use nest_error::{NestError, NestResult};

impl From<LibraryError> for NestError {
    fn from(error: LibraryError) -> Self {
        NestError::data(error.message())
            .with_code(error.nest_code())
            .with_module("nest-media-library")
            .with_source(error)
    }
}

#[cfg(test)]
mod tests {
    use nest_error::NestErrorKind;

    use super::*;

    #[test]
    fn library_error_converts_to_nest_error() {
        let library_error = LibraryError::scan("walk failed");
        let nest_error: NestError = library_error.into();
        assert_eq!(nest_error.kind(), NestErrorKind::Data);
        assert_eq!(
            nest_error.code(),
            Some(crate::codes::NEST_MEDIA_LIBRARY_SCAN_FAILED)
        );
    }
}
