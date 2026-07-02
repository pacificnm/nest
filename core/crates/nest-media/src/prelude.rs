//! Common nest-media imports.

pub use crate::artwork::{Artwork, ArtworkKind, ArtworkSource};
pub use crate::error::{MediaError, MediaErrorKind, MediaResult};
pub use crate::external::ExternalIds;
pub use crate::id::{ExternalMediaId, MediaId};
pub use crate::inspection::{MediaInput, MediaInspection};
pub use crate::item::MediaItem;
pub use crate::kind::MediaKind;
pub use crate::metadata::{MovieMetadata, MovieSearchQuery, MovieSearchResult};
pub use crate::movie::{Movie, PersonCredit};
pub use crate::tracks::{AudioTrack, HdrFormat, MediaTracks, SubtitleTrack, VideoTrack};

#[cfg(feature = "async")]
pub use crate::provider::{MediaInspector, MediaLibraryRepository, MetadataProvider};

pub use nest_error::{NestError, NestResult};
