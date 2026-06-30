# nest-airtable v1 Implementation Plan

## Status: Implemented

See [nest-airtable docs](../nest-airtable/README.md).

## Context

Airtable REST API client for Nest applications. Offset-based pagination stays in this crate — not a generic `nest-data-pagination` module.

## Crate boundaries

| Crate | Role |
|-------|------|
| `nest-airtable` | `AirtableClient`, `AirtableModule`, batch/pager/retry |
| `nest-http-client` | HTTP transport |
| `nest-config` | `[airtable]` configuration sections |
| Host | Tokio runtime, logging init |

Auth uses `Authorization: Bearer <token>` only (no legacy API-key query params).

## Deferred

- `nest-page` generic pagination (revisit when multiple integrations share patterns)
- Airtable create/delete record endpoints
- Webhook / sync orchestration crate
