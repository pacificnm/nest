# nest-stream

Byte-range file streaming for the [Nest framework](../../README.md).

**Crate path (planned):** [`core/crates/nest-stream`](../../core/crates/nest-stream) or [`modules/crates/nest-stream`](../../modules/crates/nest-stream) — TBD at implementation.

## Status

**Planned** — placeholder only. See [implementation plan](../plan/nest-stream-v1.md).

Loon v0.1 implements streaming in-app first; this crate extracts the proven handler.

## Role

`nest-stream` answers **how do we stream a scoped local file over HTTP with byte-range support?** It does not resolve slugs, scan libraries, or transcode.

| Layer | Responsibility |
|-------|----------------|
| **`nest-stream`** | Range parsing, partial content responses, MIME sniff by extension |
| `nest-file` | Scoped path open + metadata (size) |
| `nest-http-serve` | Route mounting; may integrate `StreamHandler` helper |
| Loon / apps | Slug → path resolution, auth, playback policy |

## Related

- [Implementation plan](../plan/nest-stream-v1.md)
- [nest-http-serve](../nest-http-serve/README.md) — defers file streaming in v0.1
- [nest-file](../nest-file/README.md) — scoped I/O
- [Loon stream spec](../../apps/loon/docs/v1.md#stream-handler-specification)
