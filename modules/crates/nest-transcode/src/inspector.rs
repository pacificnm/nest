//! nest-media MediaInspector implementation.

use async_trait::async_trait;
use nest_file::FileService;
use nest_media::{MediaError, MediaInput, MediaInspection, MediaInspector, MediaResult};
use tracing::instrument;

use crate::error::transcode_to_media_error;
use crate::mapper;
use crate::runner::FfprobeRunner;

/// FFprobe-backed media inspector for nest-media.
#[derive(Clone)]
pub struct FfprobeMediaInspector {
    files: FileService,
    runner: FfprobeRunner,
}

impl FfprobeMediaInspector {
    /// Creates an inspector with file service and ffprobe runner.
    pub fn new(files: FileService, runner: FfprobeRunner) -> Self {
        Self { files, runner }
    }

    /// Returns the underlying ffprobe runner.
    pub fn runner(&self) -> &FfprobeRunner {
        &self.runner
    }

    fn resolve_absolute_path(&self, path: &str) -> MediaResult<std::path::PathBuf> {
        self.files
            .metadata(path)
            .map(|metadata| metadata.path)
            .map_err(|error| MediaError::invalid_input(error.to_string()))
    }
}

#[async_trait]
impl MediaInspector for FfprobeMediaInspector {
    #[instrument(skip(self, input))]
    async fn inspect(&self, input: MediaInput) -> MediaResult<MediaInspection> {
        match input {
            MediaInput::LocalPath(path) => {
                let absolute = self.resolve_absolute_path(&path)?;
                let runner = self.runner.clone();
                let output = tokio::task::spawn_blocking(move || runner.probe_file(&absolute))
                    .await
                    .map_err(|error| {
                        MediaError::inspection(format!("ffprobe task failed: {error}"))
                    })?
                    .map_err(transcode_to_media_error)?;
                Ok(mapper::to_inspection(&output))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use nest_file::{FileModule, FileServiceConfig};
    use tempfile::tempdir;

    use super::*;
    use crate::config::TranscodeConfig;

    #[tokio::test]
    async fn resolves_relative_path_via_file_service() {
        let dir = tempdir().unwrap();
        std::fs::write(dir.path().join("clip.mkv"), b"not real video").unwrap();

        let built = nest_core::AppBuilder::new()
            .module(FileModule::with_config(
                FileServiceConfig::scoped(dir.path()).allow_create_dirs(true),
            ))
            .build()
            .unwrap();
        let files = built.context.service::<nest_file::FileService>().unwrap().clone();

        let inspector = FfprobeMediaInspector::new(
            files,
            FfprobeRunner::new(TranscodeConfig::builder().build().unwrap()).unwrap(),
        );

        let absolute = inspector.resolve_absolute_path("clip.mkv").unwrap();
        assert!(absolute.ends_with("clip.mkv"));
    }
}
