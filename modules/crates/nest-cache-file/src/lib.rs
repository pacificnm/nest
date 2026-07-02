//! Disk-backed cache adapter for the Nest cache layer.

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

pub mod adapter;
pub mod config;
pub mod index;
pub mod meta;
pub mod module;

pub use adapter::{set_with_content_type, FileCacheAdapter};
pub use config::FileCacheConfig;
pub use meta::FileCacheMeta;
pub use module::{FileCacheModule, FILE_CACHE_MODULE_ID};

pub use nest_cache::prelude::*;
pub use nest_core::{Module, ModuleId};
pub use nest_error::{NestError, NestResult};
