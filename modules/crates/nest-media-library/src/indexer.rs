//! Library indexing orchestration.

use std::sync::Arc;

use nest_media::{
    MediaId, MediaInput, MediaInspector, MediaLibraryRepository, MetadataProvider,
    MovieSearchQuery,
};
use tracing::instrument;

use crate::config::MediaLibraryConfig;
use crate::error::{LibraryError, LibraryResult};
use crate::scan::{
    LibraryScanOptions, LibraryScanner, ScanItemStatus, ScanResult,
};
use crate::scan::stats::record_error;

/// Orchestrates filesystem discovery and optional provider enrichment.
#[derive(Clone)]
pub struct LibraryIndexer {
    scanner: LibraryScanner,
    metadata: Option<Arc<dyn MetadataProvider>>,
    inspector: Option<Arc<dyn MediaInspector>>,
    repository: Option<Arc<dyn MediaLibraryRepository>>,
}

impl LibraryIndexer {
    /// Creates an indexer with discovery-only defaults.
    pub fn new(scanner: LibraryScanner) -> Self {
        Self {
            scanner,
            metadata: None,
            inspector: None,
            repository: None,
        }
    }

    /// Attaches a metadata provider.
    pub fn with_metadata(mut self, metadata: Arc<dyn MetadataProvider>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Attaches a media inspector.
    pub fn with_inspector(mut self, inspector: Arc<dyn MediaInspector>) -> Self {
        self.inspector = Some(inspector);
        self
    }

    /// Attaches a media library repository.
    pub fn with_repository(mut self, repository: Arc<dyn MediaLibraryRepository>) -> Self {
        self.repository = Some(repository);
        self
    }

    /// Runs the scan pipeline with the given options.
    #[instrument(skip(self, config), fields(library_id = %config.id))]
    pub async fn scan_library(
        &self,
        config: &MediaLibraryConfig,
        options: LibraryScanOptions,
    ) -> LibraryResult<ScanResult> {
        let scanner = self.scanner.clone();
        let config = config.clone();
        let mut result = tokio::task::spawn_blocking(move || scanner.discover(&config))
            .await
            .map_err(|error| {
                LibraryError::scan(format!("scan task join failed: {error}"))
            })??;

        if options.inspect_files {
            self.inspect_candidates(&mut result).await?;
        }

        if options.fetch_metadata {
            self.fetch_metadata(&mut result).await?;
        }

        if options.persist {
            self.persist_candidates(&mut result).await?;
        }

        Ok(result)
    }

    async fn inspect_candidates(&self, result: &mut ScanResult) -> LibraryResult<()> {
        let Some(inspector) = &self.inspector else {
            return Ok(());
        };

        for candidate in &mut result.candidates {
            let path = candidate.file.relative_path.clone();
            match inspector
                .inspect(MediaInput::LocalPath(path.clone()))
                .await
            {
                Ok(inspection) => candidate.inspection = Some(inspection),
                Err(error) => {
                    candidate.status = ScanItemStatus::Error;
                    record_error(result, &path, error.message());
                }
            }
        }

        Ok(())
    }

    async fn fetch_metadata(&self, result: &mut ScanResult) -> LibraryResult<()> {
        let Some(provider) = &self.metadata else {
            return Ok(());
        };

        for candidate in &mut result.candidates {
            let path = candidate.file.relative_path.clone();
            let title = candidate
                .guessed_title
                .clone()
                .unwrap_or_else(|| path.clone());

            let mut query = MovieSearchQuery::new(title);
            if let Some(year) = candidate.guessed_year {
                query = query.with_year(year);
            }

            match provider.search_movie(query).await {
                Ok(results) if !results.is_empty() => {
                    let external_id = results[0].external_id.clone();
                    match provider.get_movie(external_id).await {
                        Ok(metadata) => candidate.metadata = Some(metadata),
                        Err(error) => {
                            candidate.status = ScanItemStatus::Error;
                            record_error(result, &path, error.message());
                        }
                    }
                }
                Ok(_) => {
                    record_error(
                        result,
                        &path,
                        "metadata provider returned no search results",
                    );
                }
                Err(error) => {
                    candidate.status = ScanItemStatus::Error;
                    record_error(result, &path, error.message());
                }
            }
        }

        Ok(())
    }

    async fn persist_candidates(&self, result: &mut ScanResult) -> LibraryResult<()> {
        let Some(repository) = &self.repository else {
            return Ok(());
        };

        for candidate in &mut result.candidates {
            let path = candidate.file.relative_path.clone();
            let movie = if let Some(metadata) = candidate.metadata.clone() {
                metadata.into_movie(media_id_for_path(&path))
            } else {
                let title = candidate
                    .guessed_title
                    .clone()
                    .unwrap_or_else(|| path.clone());
                let mut movie = nest_media::Movie::new(media_id_for_path(&path), title);
                movie.year = candidate.guessed_year;
                movie
            };

            match repository.save_movie(movie).await {
                Ok(()) => candidate.status = ScanItemStatus::New,
                Err(error) => {
                    candidate.status = ScanItemStatus::Error;
                    record_error(result, &path, error.message());
                }
            }
        }

        Ok(())
    }
}

fn media_id_for_path(path: &str) -> MediaId {
    MediaId::new(format!("file:{path}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use nest_media::{
        MediaResult, Movie, MovieMetadata, MovieSearchResult, MovieSearchQuery, ExternalMediaId,
    };
    use nest_file::{FileService, FileServiceConfig};
    use std::fs;
    use tempfile::tempdir;

    struct MockProvider;

    #[async_trait]
    impl MetadataProvider for MockProvider {
        async fn search_movie(
            &self,
            query: MovieSearchQuery,
        ) -> MediaResult<Vec<MovieSearchResult>> {
            Ok(vec![MovieSearchResult {
                external_id: ExternalMediaId::new("tmdb:1"),
                title: query.query,
                year: query.year,
                summary: None,
            }])
        }

        async fn get_movie(&self, id: ExternalMediaId) -> MediaResult<MovieMetadata> {
            Ok(MovieMetadata {
                external_id: id,
                title: "Alien".into(),
                original_title: None,
                sort_title: None,
                year: Some(1979),
                runtime_seconds: None,
                rating: None,
                summary: None,
                genres: Vec::new(),
                cast: Vec::new(),
                crew: Vec::new(),
                tracks: nest_media::MediaTracks::new(),
                external_ids: nest_media::ExternalIds::new(),
            })
        }
    }

    struct MockRepository {
        saved: std::sync::Mutex<Vec<Movie>>,
    }

    impl MockRepository {
        fn new() -> Self {
            Self {
                saved: std::sync::Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl MediaLibraryRepository for MockRepository {
        async fn save_movie(&self, movie: Movie) -> MediaResult<()> {
            self.saved.lock().unwrap().push(movie);
            Ok(())
        }

        async fn get_movie(
            &self,
            _id: nest_media::MediaId,
        ) -> MediaResult<Option<Movie>> {
            Ok(None)
        }

        async fn list_movies(&self) -> MediaResult<Vec<Movie>> {
            Ok(self.saved.lock().unwrap().clone())
        }
    }

    #[tokio::test]
    async fn indexer_persists_with_mock_provider_and_repository() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("Alien (1979).mkv"), b"video").unwrap();

        let files = FileService::with_config(
            FileServiceConfig::scoped(dir.path()).allow_create_dirs(true),
        )
        .unwrap();
        let scanner = LibraryScanner::new(files);
        let repository = Arc::new(MockRepository::new());
        let indexer = LibraryIndexer::new(scanner)
            .with_metadata(Arc::new(MockProvider))
            .with_repository(repository.clone());

        let config = MediaLibraryConfig::new("main", ["."]);
        let result = indexer
            .scan_library(&config, LibraryScanOptions::full())
            .await
            .unwrap();

        assert_eq!(result.candidates.len(), 1);
        assert_eq!(repository.saved.lock().unwrap().len(), 1);
        assert_eq!(repository.saved.lock().unwrap()[0].title, "Alien");
    }
}
