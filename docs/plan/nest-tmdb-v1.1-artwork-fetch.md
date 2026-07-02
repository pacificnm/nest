# nest-tmdb v1.1 — MovieFetchResult (artwork paths)

## Status: Planned

Follow-up to [nest-tmdb v1](nest-tmdb-v1.md). Unblocks [Loon enrichment](../../apps/loon/docs/v1.md#artwork--enrichment-strategy) without a second TMDB HTTP round-trip.

## Problem

`TmdbMetadataProvider::get_movie` returns `MovieMetadata`, which **omits** `poster_path` and `backdrop_path`. Loon must store path tokens for SQLite and rebuild URLs with `TmdbImageService`.

Today Loon would need a duplicate `movie_details` call or fork DTO mapping.

## Decision

Add a **nest-tmdb-native** fetch type — not a change to `MetadataProvider` trait (keep provider contract minimal).

```rust
/// Full movie fetch including TMDB artwork path tokens.
pub struct MovieFetchResult {
    pub metadata: MovieMetadata,
    pub poster_path: Option<String>,
    pub backdrop_path: Option<String>,
}

impl TmdbMetadataProvider {
    /// Fetches movie metadata + artwork paths in one TMDB round-trip set
    /// (details + credits + external_ids — same as get_movie today).
    pub async fn fetch_movie(&self, id: ExternalMediaId) -> MediaResult<MovieFetchResult>;
}
```

`get_movie` remains and delegates:

```rust
async fn get_movie(&self, id: ExternalMediaId) -> MediaResult<MovieMetadata> {
    Ok(self.fetch_movie(id).await?.metadata)
}
```

## Mapping

Reuse existing client calls in `provider.rs`:

```rust
let movie = self.client.movie_details(movie_id).await?;
let (poster_path, backdrop_path) = artwork_paths(&movie);
let metadata = map_movie_metadata(movie, credits, external_ids);
Ok(MovieFetchResult { metadata, poster_path, backdrop_path })
```

Export `artwork_paths` from `mapper` (already exists — ensure `pub`).

## Loon usage

```rust
// services/enrichment.rs
let fetch = tmdb.fetch_movie(external_id).await?;
candidate.metadata = Some(fetch.metadata);
candidate.poster_path = fetch.poster_path;
candidate.backdrop_path = fetch.backdrop_path;
```

Or extend `EnrichedCandidate` in Loon only — no nest-media-library change required.

## Alternative considered

Add `artwork: Vec<Artwork>` to `MovieMetadata` in nest-media — rejected for v1.1 because:

- Provider trait change affects all implementors
- Artwork URLs depend on `TmdbImageService` base URL (app-layer concern)
- Path **tokens** belong in fetch result; URLs built at API boundary

Revisit `MovieMetadata.artwork` in nest-media v0.2 if multiple providers need it.

## Scope

| Ship | Defer |
|------|-------|
| `MovieFetchResult` | `append_to_response` batching |
| `fetch_movie` on provider | TV show fetch result |
| Unit test with fixture JSON | Image list endpoint |

## Related

- [nest-tmdb v1](nest-tmdb-v1.md)
- [Loon v1 plan](../../apps/loon/docs/v1.md)
