# nest-file-csv

CSV import and export for the [Nest framework](../../README.md).

**Crate path:** [`core/crates/nest-file-csv`](../../core/crates/nest-file-csv)

Depends on [`nest-file`](../nest-file/README.md) for all file I/O.

## Quick start

```rust
use nest_core::AppBuilder;
use nest_file::{FileModule, FileServiceConfig};
use nest_file_csv::{CsvModule, CsvOptions, CsvService};

let built = AppBuilder::new()
    .module(FileModule::with_config(
        FileServiceConfig::scoped("./workspace").allow_create_dirs(true),
    ))
    .module(CsvModule)
    .build()?;

let csv = built.context.service::<CsvService>()?;

#[derive(serde::Deserialize)]
struct ImportRow {
    customer_id: String,
    email: String,
}

let options = CsvOptions::default()
    .map_column("cust_id", "customer_id")
    .require_columns(["customer_id", "email"]);

let rows: Vec<ImportRow> = csv.read_typed("imports/customers.csv", &options)?;
```

## Column mapping

Headers are trimmed and lowercased for lookup (original names appear in issues). Map source headers to target struct field names:

```rust
let options = CsvOptions::default()
    .map_column("cust_id", "customer_id")
    .map_column("eml", "email");
```

Missing required columns fail before row processing with `NEST_CSV_REQUIRED_COLUMN_MISSING`.

## Continue on error

```rust
let options = CsvOptions::default().continue_on_error(true);
let report = csv.read_typed_report::<ImportRow>("mixed.csv", &options)?;

println!("valid: {}, skipped: {}", report.valid_rows, report.skipped_rows);
for issue in &report.issues {
    eprintln!("row {}: {} — {}", issue.row_number, issue.code, issue.message);
}
```

Use `max_errors(n)` to stop after a threshold of row issues.

## Writing

```rust
csv.write_typed("exports/customers.csv", &rows, &CsvOptions::default())?;
```

`create_parent_dirs` (default `true`) passes through to `FileService` write options.

## Validation feature

Enable `validate` to run [`nest-validation`](../nest-validation/README.md) `Validate` on deserialized rows:

```toml
nest-file-csv = { path = "../nest-file-csv", features = ["validate"] }
```

```rust
let report = csv.read_typed_with_validation::<ImportRow>("data.csv", &options)?;
```

Custom row checks without the feature use `CsvRowValidator` via `read_typed_report_with_validator`.

## Error codes

| Code | When |
|------|------|
| `NEST_CSV_PARSE_FAILED` | CSV parse error |
| `NEST_CSV_DESERIALIZE_FAILED` | Row → struct failed |
| `NEST_CSV_REQUIRED_COLUMN_MISSING` | Header check failed |
| `NEST_CSV_VALIDATION_FAILED` | Row `Validate` failed |
| `NEST_CSV_WRITE_FAILED` | Serialize/write error |
| `NEST_CSV_ROW_LIMIT_EXCEEDED` | `max_errors` threshold hit |

File-level errors propagate from `nest-file` unchanged.

## Related

- [nest-file](../nest-file/README.md) — underlying file service
- [Implementation plan](../plan/nest-file-csv-v1.md)
