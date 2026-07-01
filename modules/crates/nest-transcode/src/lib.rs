//! FFprobe media inspection for the Nest framework.
//!
//! `nest-transcode` probes local media files with FFprobe and implements
//! [`nest_media::MediaInspector`]. v0.1 is inspection-only; FFmpeg transcode
//! jobs are deferred to v0.2.
//!
//! # Quick start
//!
//! ```no_run
//! use nest_core::AppBuilder;
//! use nest_file::FileModule;
//! use nest_media::{MediaInput, MediaInspector};
//! use nest_transcode::{FfprobeMediaInspector, TranscodeConfig, TranscodeModule};
//!
//! let built = AppBuilder::new()
//!     .module(FileModule::scoped("./media"))
//!     .module(TranscodeModule::with_config(
//!         TranscodeConfig::builder().build().unwrap(),
//!     ))
//!     .build()
//!     .unwrap();
//!
//! let inspector = built.context.service::<FfprobeMediaInspector>().unwrap();
//! # async fn demo(inspector: &FfprobeMediaInspector) -> nest_media::MediaResult<()> {
//! let inspection = inspector
//!     .inspect(MediaInput::LocalPath("Movies/Alien.mkv".into()))
//!     .await?;
//! # Ok(())
//! # }
//! ```

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

mod codes;
mod config;
mod dto;
mod error;
mod inspector;
mod mapper;
mod module;
mod runner;

pub mod prelude;

pub use config::{
    resolve_ffprobe_path, TranscodeConfig, TranscodeConfigBuilder, DEFAULT_FFPROBE_PATH,
    DEFAULT_FFPROBE_PATH_ENV, DEFAULT_TIMEOUT_SECONDS,
};
pub use error::{transcode_to_media_error, TranscodeError, TranscodeErrorKind, TranscodeResult};
pub use inspector::FfprobeMediaInspector;
pub use module::{TranscodeModule, TRANSCODE_MODULE_ID};
pub use runner::FfprobeRunner;

pub use nest_error::{NestError, NestResult};
pub use nest_media::{MediaInput, MediaInspection, MediaInspector};

impl From<TranscodeError> for NestError {
    fn from(error: TranscodeError) -> Self {
        NestError::task(error.message())
            .with_code(error.nest_code())
            .with_module("nest-transcode")
            .with_source(error)
    }
}

#[cfg(test)]
mod lib_tests {
    use nest_error::NestErrorKind;

    use super::*;

    #[test]
    fn transcode_error_converts_to_nest_error() {
        let error = TranscodeError::probe("ffprobe failed");
        let nest_error: NestError = error.into();
        assert_eq!(nest_error.kind(), NestErrorKind::Task);
        assert_eq!(
            nest_error.code(),
            Some(codes::NEST_TRANSCODE_PROBE_FAILED)
        );
    }
}
