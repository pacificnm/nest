//! Remote image fetch and cache for Nest applications.
//!
//! Uses [`nest_cache::Cache`] (typically [`nest_cache_file::FileCacheAdapter`] on disk)
//! to store bytes fetched over HTTP. Desktop apps display cached bytes in the React
//! webview via Tauri IPC (see `docs/plan/nest-react-ui-v1.md` in the Nest repo).

#![warn(missing_docs)]

mod key;
mod mime;
mod module;
mod service;
mod url;

pub use module::{ImageModule, IMAGE_MODULE_ID};

pub use key::{artwork_tags, cache_key_for_url, movie_cache_tag, profile_tags};
pub use mime::detect_mime;
pub use service::ImageService;
pub use url::resolve_url;
