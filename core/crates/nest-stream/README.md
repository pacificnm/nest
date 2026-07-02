# nest-stream

**Status: Planned — not implemented yet.**

Placeholder crate directory for HTTP byte-range file streaming.

## Intent

Serve local media files with `Accept-Ranges: bytes` support — extracted from Loon once the stream handler stabilizes.

## Documentation

| Document | Description |
|----------|-------------|
| [docs/nest-stream/README.md](../../../docs/nest-stream/README.md) | Overview |
| [docs/plan/nest-stream-v1.md](../../../docs/plan/nest-stream-v1.md) | Implementation plan |

## Revisit when

- Loon `services/streaming.rs` proves the range/Content-Type behavior ([Loon v1 plan](../../../apps/loon/docs/v1.md))
- Other Nest apps need scoped file streaming without reimplementing RFC 7233 parsing

No `Cargo.toml` or source yet.
