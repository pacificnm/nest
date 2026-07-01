//! Provider trait contracts.

#[cfg(feature = "async")]
mod inspector;
#[cfg(feature = "async")]
mod metadata;
#[cfg(feature = "async")]
mod repository;

#[cfg(feature = "async")]
pub use inspector::MediaInspector;
#[cfg(feature = "async")]
pub use metadata::MetadataProvider;
#[cfg(feature = "async")]
pub use repository::MediaLibraryRepository;
