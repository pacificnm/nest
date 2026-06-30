# Core framework crates

Stable Nest framework crates live here. These define hosts, contracts, and shared infrastructure.

Do not add application-specific or third-party integration logic here — use `modules/crates/` instead.

**Dependency rule:** core crates must not depend on `modules/` or `apps/`. See [docs/architecture.md](../../docs/architecture.md).
