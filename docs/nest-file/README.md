# nest-file

Sync file I/O for the [Nest framework](../../README.md).

**Crate path:** [`core/crates/nest-file`](../../core/crates/nest-file)

## Quick start

```rust
use nest_core::AppBuilder;
use nest_file::{FileModule, FileService, FileServiceConfig};

let mut built = AppBuilder::new()
    .module(FileModule::scoped("./workspace"))
    .build()?;
built.startup()?;

let files = built.context.service::<FileService>()?;
let text = files.read_text("config/app.toml")?;
files.write_text("output/report.txt", text)?;
```

## Scoped vs unscoped

| Mode | Config | Use case |
|------|--------|----------|
| Unscoped | `FileModule::default()` | Admin tools, explicit absolute paths |
| Scoped | `FileModule::scoped(root)` | Workspace / app data directory |

Scoped mode rejects absolute paths, `..` traversal, and symlink escapes outside the root.

## Write options

```rust
use nest_file::WriteOptions;

let options = WriteOptions::from_config(files.config())
    .atomic()
    .backup()
    .create_parents();

files.write_bytes_with_options("data/export.bin", bytes, options)?;
```

## Large files

nest-file is sync. For large reads/writes, run operations inside [`nest-task-runtime`](../nest-task-runtime/README.md) `spawn_blocking`.

## Related

- [nest-file-csv](../nest-file-csv/README.md) — CSV import/export with column mapping and typed rows
- [nest-file-csv plan](../plan/nest-file-csv-v1.md)
