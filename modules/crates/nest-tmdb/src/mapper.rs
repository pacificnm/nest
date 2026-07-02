//! TMDB DTO to nest-media mapping.

use nest_media::{
    ExternalIds, ExternalMediaId, MediaError, MediaResult, MediaTracks, MovieMetadata,
    MovieSearchResult, PersonCredit,
};

use crate::dto::credits::{CastMember, CrewMember, MovieCreditsResponse};
use crate::dto::external_ids::MovieExternalIdsResponse;
use crate::dto::movie::MovieDetailsResponse;
use crate::dto::search::MovieSearchResultItem;

/// Maps TMDB search hits to nest-media search results.
pub fn map_search_results(items: Vec<MovieSearchResultItem>) -> Vec<MovieSearchResult> {
    items.into_iter().map(map_search_result).collect()
}

/// Maps one TMDB search hit.
pub fn map_search_result(item: MovieSearchResultItem) -> MovieSearchResult {
    MovieSearchResult {
        external_id: external_id_for_movie(item.id),
        title: item.title,
        year: year_from_release_date(item.release_date.as_deref()),
        summary: item.overview,
    }
}

/// Maps TMDB movie details, credits, and external ids to [`MovieMetadata`].
pub fn map_movie_metadata(
    movie: MovieDetailsResponse,
    credits: MovieCreditsResponse,
    external_ids: MovieExternalIdsResponse,
) -> MovieMetadata {
    let mut ids = ExternalIds::new();
    ids.tmdb_id = Some(movie.id.to_string());
    if let Some(imdb_id) = external_ids.imdb_id.filter(|id| !id.is_empty()) {
        ids.imdb_id = Some(imdb_id);
    }

    MovieMetadata {
        external_id: external_id_for_movie(movie.id),
        title: movie.title,
        original_title: movie.original_title,
        sort_title: None,
        year: year_from_release_date(movie.release_date.as_deref()),
        runtime_seconds: movie.runtime.map(|minutes| minutes * 60),
        rating: None,
        summary: movie.overview,
        genres: movie.genres.into_iter().map(|genre| genre.name).collect(),
        cast: map_cast(credits.cast),
        crew: map_crew(credits.crew),
        tracks: MediaTracks::new(),
        external_ids: ids,
    }
}

/// Returns poster and backdrop paths from movie details.
pub fn artwork_paths(movie: &MovieDetailsResponse) -> (Option<String>, Option<String>) {
    (movie.poster_path.clone(), movie.backdrop_path.clone())
}

/// Parses a nest-media external id (`tmdb:{id}`) into a TMDB movie id.
pub fn parse_movie_external_id(id: &ExternalMediaId) -> MediaResult<u32> {
    let value = id.as_str().strip_prefix("tmdb:").ok_or_else(|| {
        MediaError::invalid_input(format!(
            "expected TMDB external id prefix tmdb:, got {}",
            id.as_str()
        ))
    })?;

    value
        .parse::<u32>()
        .map_err(|_| MediaError::invalid_input(format!("invalid TMDB movie id: {value}")))
}

/// Formats a TMDB movie id as a nest-media external id.
pub fn external_id_for_movie(id: u32) -> ExternalMediaId {
    ExternalMediaId::new(format!("tmdb:{id}"))
}

fn map_cast(cast: Vec<CastMember>) -> Vec<PersonCredit> {
    let mut members = cast;
    members.sort_by_key(|member| member.order.unwrap_or(i32::MAX));
    members
        .into_iter()
        .map(|member| PersonCredit {
            name: member.name,
            role: "Actor".into(),
            character: member.character.filter(|value| !value.is_empty()),
            profile_path: member
                .profile_path
                .filter(|value| !value.is_empty()),
        })
        .collect()
}

fn map_crew(crew: Vec<CrewMember>) -> Vec<PersonCredit> {
    crew.into_iter()
        .filter_map(|member| {
            let role = member.job.filter(|job| !job.is_empty())?;
            Some(PersonCredit::new(member.name, role, None))
        })
        .collect()
}

fn year_from_release_date(release_date: Option<&str>) -> Option<u16> {
    let release_date = release_date?;
    let year = release_date.get(0..4)?;
    year.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dto::credits::{CastMember, CrewMember, MovieCreditsResponse};
    use crate::dto::external_ids::MovieExternalIdsResponse;
    use crate::dto::movie::{GenreItem, MovieDetailsResponse};
    use crate::dto::search::MovieSearchResultItem;

    #[test]
    fn maps_search_result() {
        let item = MovieSearchResultItem {
            id: 348,
            title: "Alien".into(),
            overview: Some("In space no one can hear you scream.".into()),
            release_date: Some("1979-05-25".into()),
        };
        let result = map_search_result(item);
        assert_eq!(result.external_id.as_str(), "tmdb:348");
        assert_eq!(result.title, "Alien");
        assert_eq!(result.year, Some(1979));
    }

    #[test]
    fn maps_movie_metadata_with_credits_and_external_ids() {
        let movie = MovieDetailsResponse {
            id: 348,
            title: "Alien".into(),
            original_title: Some("Alien".into()),
            overview: Some("Summary".into()),
            release_date: Some("1979-05-25".into()),
            runtime: Some(117),
            poster_path: Some("/poster.jpg".into()),
            backdrop_path: Some("/backdrop.jpg".into()),
            genres: vec![GenreItem {
                name: "Horror".into(),
            }],
        };
        let credits = MovieCreditsResponse {
            cast: vec![CastMember {
                name: "Sigourney Weaver".into(),
                character: Some("Ripley".into()),
                order: Some(0),
                profile_path: Some("/profile.jpg".into()),
            }],
            crew: vec![CrewMember {
                name: "Ridley Scott".into(),
                job: Some("Director".into()),
            }],
        };
        let external_ids = MovieExternalIdsResponse {
            imdb_id: Some("tt0078748".into()),
            tmdb_id: Some(348),
        };

        let metadata = map_movie_metadata(movie, credits, external_ids);
        assert_eq!(metadata.title, "Alien");
        assert_eq!(metadata.year, Some(1979));
        assert_eq!(metadata.runtime_seconds, Some(117 * 60));
        assert_eq!(metadata.genres, vec!["Horror"]);
        assert_eq!(metadata.cast.len(), 1);
        assert_eq!(metadata.cast[0].profile_path.as_deref(), Some("/profile.jpg"));
        assert_eq!(metadata.crew[0].role, "Director");
        assert_eq!(metadata.external_ids.imdb_id.as_deref(), Some("tt0078748"));
    }

    #[test]
    fn parses_external_movie_id() {
        let id = ExternalMediaId::new("tmdb:348");
        assert_eq!(parse_movie_external_id(&id).unwrap(), 348);
    }
}
