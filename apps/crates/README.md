# Applications

Shipping Nest applications live here. Each app composes `core/crates` hosts with selected `modules/crates` integrations.

Planned: `airtable-sync-core`, `airtable-sync-cli`, `airtable-sync-gui`, `kiwi`, `finch`, …

**Dependency rule:** apps may depend on `core/` and `modules/`. Nothing in core or modules may depend on an app. See [docs/architecture.md](../../docs/architecture.md).

Add a new app with:

```bash
cargo new --lib apps/crates/my-app
```

Then register the path in the root workspace `Cargo.toml`.
