//! Cache contracts for the Nest framework.
//!
//! nest-cache defines keyed entries, optional TTL, tags for grouped invalidation,
//! and a pluggable [`CacheAdapter`] trait. Storage backends live in adapter crates.

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

pub mod adapter;
pub mod cache;
pub mod codes;
pub mod entry;
pub mod error;
pub mod key;
pub mod module;
pub mod prelude;

pub use adapter::{CacheAdapter, MemoryCacheAdapter};
pub use cache::Cache;
pub use entry::CacheEntry;
pub use error::{CacheError, CacheErrorKind, CacheResult};
pub use key::CacheKey;
pub use module::{CacheModule, CACHE_MODULE_ID};

pub use nest_core::{Module, ModuleId};
pub use nest_error::{NestError, NestResult};

impl From<CacheError> for NestError {
    fn from(error: CacheError) -> Self {
        let mut nest_error = NestError::data(error.message()).with_module("nest-cache");

        if let Some(code) = error.nest_code() {
            nest_error = nest_error.with_code(code);
        }

        nest_error.with_source(error)
    }
}
