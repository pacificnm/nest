//! Cache errors.

use std::error::Error;
use std::fmt;

use crate::codes::{
    NEST_CACHE_ADAPTER, NEST_CACHE_EXPIRED, NEST_CACHE_FAILED, NEST_CACHE_IO, NEST_CACHE_NOT_FOUND,
    NEST_CACHE_SERIALIZATION,
};

/// Result type for cache operations.
pub type CacheResult<T> = Result<T, CacheError>;

/// High-level category for a cache error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CacheErrorKind {
    /// Entry not found.
    NotFound,
    /// Entry expired.
    Expired,
    /// Adapter I/O failure.
    Io,
    /// Adapter rejected the operation.
    Adapter,
    /// Serialization failure.
    Serialization,
}

/// Structured cache error.
#[derive(Debug)]
pub struct CacheError {
    kind: CacheErrorKind,
    message: String,
    code: Option<String>,
    source: Option<Box<dyn Error + Send + Sync>>,
}

impl CacheError {
    /// Creates a new cache error.
    pub fn new(kind: CacheErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            code: None,
            source: None,
        }
    }

    /// Creates a not-found error.
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(CacheErrorKind::NotFound, message).with_code(NEST_CACHE_NOT_FOUND)
    }

    /// Creates an expired error.
    pub fn expired(message: impl Into<String>) -> Self {
        Self::new(CacheErrorKind::Expired, message).with_code(NEST_CACHE_EXPIRED)
    }

    /// Creates an I/O error.
    pub fn io(message: impl Into<String>) -> Self {
        Self::new(CacheErrorKind::Io, message).with_code(NEST_CACHE_IO)
    }

    /// Creates an adapter error.
    pub fn adapter(message: impl Into<String>) -> Self {
        Self::new(CacheErrorKind::Adapter, message).with_code(NEST_CACHE_ADAPTER)
    }

    /// Creates a serialization error.
    pub fn serialization(message: impl Into<String>) -> Self {
        Self::new(CacheErrorKind::Serialization, message).with_code(NEST_CACHE_SERIALIZATION)
    }

    /// Sets a stable error code.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Attaches a source error.
    pub fn with_source(mut self, source: impl Error + Send + Sync + 'static) -> Self {
        self.source = Some(Box::new(source));
        self
    }

    /// Returns the error kind.
    pub fn kind(&self) -> CacheErrorKind {
        self.kind
    }

    /// Returns the human-readable message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Returns the stable error code when set.
    pub fn nest_code(&self) -> Option<&str> {
        self.code.as_deref()
    }
}

impl fmt::Display for CacheError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl Error for CacheError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|error| error.as_ref() as _)
    }
}

impl Default for CacheError {
    fn default() -> Self {
        Self::new(CacheErrorKind::Adapter, "cache operation failed").with_code(NEST_CACHE_FAILED)
    }
}
