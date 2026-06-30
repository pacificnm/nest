# nest-file v1 Implementation Plan

## Status: Implemented

See [nest-file docs](../nest-file/README.md).

## Context

Sync-first file I/O with centralized `SafePathResolver`. Format parsers (CSV, JSON, etc.) are separate crates.

## Crate boundaries

| Crate | Role |
|-------|------|
| `nest-file` | `FileService`, `SafePathResolver`, scoped/unscoped modes |
| `nest-file-csv` | Implemented — depends on nest-file |

## nest-file

- `FileModule`, `FileService`, `FileServiceConfig`, `WriteOptions`
- `SafePathResolver` — all operations go through path resolution
- Scoped mode: root set, no absolute paths, no traversal, no symlink escape
- Atomic write + `.bak` backup options
- `FileError` + `From<FileError> for NestError`
- `tracing` instrumentation

## v1 limitations

- Sync only (use `nest-task-runtime::spawn_blocking` for large I/O)
- No watchers, cloud storage, glob, compression, streaming

## Follow-up

- Streaming large files
- Additional format crates (`nest-file-excel`, etc.)
