//! Remote image fetch and cache for Nest desktop apps.
//!
//! Uses [`nest_cache::Cache`] (typically [`nest_cache_file::FileCacheAdapter`] on disk)
//! to store bytes fetched over HTTP. egui widgets decode cached bytes into textures.

#![warn(missing_docs)]

mod decode;
mod key;
mod module;
mod service;
mod url;

#[cfg(feature = "egui")]
mod widget;

pub use module::{ImageModule, IMAGE_MODULE_ID};

pub use key::{artwork_tags, cache_key_for_url, movie_cache_tag, profile_tags};
pub use service::ImageService;
pub use url::resolve_url;

#[cfg(feature = "egui")]
pub use widget::{invalidate_movie_images, RemoteImage};
