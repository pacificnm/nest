//! Common imports for nest-cache consumers.

pub use crate::adapter::{CacheAdapter, MemoryCacheAdapter};
pub use crate::cache::Cache;
pub use crate::entry::CacheEntry;
pub use crate::error::{CacheError, CacheErrorKind, CacheResult};
pub use crate::key::CacheKey;
pub use crate::module::{CacheModule, CACHE_MODULE_ID};

pub use nest_core::{Module, ModuleId};
pub use nest_error::{NestError, NestResult};
