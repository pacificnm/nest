# nest-file-csv v1 Implementation Plan

## Status: Implemented

See [nest-file-csv docs](../nest-file-csv/README.md).

## Context

First format-specific file crate: CSV read/write/import on top of [`nest-file`](../nest-file/README.md), with column mapping, required columns, continue-on-error reports, optional `nest-validation` integration, and tracing.

## Crate boundaries

| Crate | Role |
|-------|------|
| `nest-file` | `FileService`, path safety, atomic write |
| `nest-file-csv` | `CsvService`, mapping, typed import/export, reports |
| `nest-validation` (optional) | `Validate` trait integration via `validate` feature |

## nest-file-csv

- `CsvModule` depends on `FileModule` / `FILE_MODULE_ID`
- `CsvService` — all I/O through `FileService`
- `CsvOptions` — delimiter, trim, column mapping, required columns, `continue_on_error`, `max_errors`
- `CsvReadReport` / `CsvWriteReport` / `CsvRowIssue`
- `CsvRowValidator` trait for custom row checks
- `read_typed_with_validation` when `validate` feature enabled
- `CsvError` + `NEST_CSV_*` codes via `nest-error`

## v1 limitations

- Sync, in-memory parsing (full file loaded as text)
- No streaming, async CSV, Excel, schema inference, or Airtable API

## Follow-up

- Streaming large CSV (nest-file-csv v2 + nest-file streaming)
- `nest-file-excel`, `nest-airtable`
