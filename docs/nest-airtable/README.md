# nest-airtable

Airtable REST API client for the [Nest framework](../../README.md).

**Crate path:** [`modules/crates/nest-airtable`](../../modules/crates/nest-airtable)

Provides [`AirtableClient`] for listing and updating records with Airtable-specific offset pagination, batch chunking (10 records per request), and rate-limit retry handling. Uses [`nest_http_client::HttpClientService`] for transport.

## Quick start

```rust
use nest_airtable::{AirtableListParams, AirtableModule};
use nest_core::AppBuilder;
use nest_http_client::HttpClientModule;

#[tokio::main]
async fn main() -> nest_error::NestResult<()> {
    let built = AppBuilder::new()
        .module(HttpClientModule::default())
        .module(AirtableModule::new()) // loads [airtable] from ConfigService
        .build()?;

    let airtable = built.context.service::<nest_airtable::AirtableClient>()?;
    let records = airtable
        .list_all_records("assets", AirtableListParams::default())
        .await?;
    println!("loaded {} records", records.len());
    Ok(())
}
```

## Configuration

```toml
[airtable]
api_url = "https://api.airtable.com/v0"
token_env = "AIRTABLE_TOKEN"
base_id = "appXXXXXXXXXXXXXX"

[airtable.tables.assets]
table_id = "tblXXXXXXXXXXXXXX"
primary_key_field = "Asset ID"
```

Export the token before running:

```bash
export AIRTABLE_TOKEN="pat..."
```

Logical table names (e.g. `assets`) map to Airtable table ids in config.

## Client API

| Method | Description |
|--------|-------------|
| `list_records_page` | One page using `pageSize` (default 100) and optional offset |
| `list_all_records` | Follows offset tokens until exhausted |
| `update_record` | PATCH a single record by id |
| `batch_update_records` | PATCH up to 10 per request; auto-chunks larger sets |

```rust
let airtable = ctx.service::<AirtableClient>()?;

let page = airtable
    .list_records_page("assets", &AirtableListParams::default())
    .await?;

let all = airtable
    .list_all_records("assets", AirtableListParams::default())
    .await?;

airtable
    .batch_update_records("assets", updates)
    .await?;
```

## Pagination

[`AirtablePager`] exposes explicit page iteration. [`AirtableListParams`] supports `view`, `filter_by_formula`, `fields`, and cooperative cancellation via [`nest_task::CancelToken`].

## Batch limits

[`AirtableBatch`] chunks updates into groups of 10 (Airtable API limit). `batch_update_records` calls the API once per chunk.

## Retry and rate limits

[`AirtableRetryPolicy`] retries timeouts, connection errors, HTTP 503, and 429 responses with exponential backoff. [`AirtableRateLimitHook`] adds an optional delay between paginated requests.

## Module dependencies

`AirtableModule` requires `nest-http-client`. When using `AirtableModule::new()`, register `ConfigService` before building so `[airtable]` can be loaded.

## Related docs

- [Implementation plan](../plan/nest-airtable-v1.md)
- [nest-http-client](../nest-http-client/README.md)
- [nest-config](../nest-config/README.md)
