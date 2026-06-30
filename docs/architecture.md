# Nest repository architecture

Nest is organized in three layers. Each layer has a fixed place in the tree and fixed dependency rules.

## Layers

```text
Apps          apps/<product>/    shipping products (airtable-sync, kiwi, …)
  │
  ▼
Modules       modules/crates/    optional integrations (nest-airtable, nest-data-sqlite, …)
  │
  ▼
Core          core/crates/       framework hosts and contracts (nest-core, nest-cli, …)
```

Within **core**, the runtime stack flows:

```text
nest-core → nest-app → nest-cli / nest-tui / nest-gui
```

Hosts own presentation (CLI parsing, event loops, logging init). Modules register services into the Nest container. Apps wire hosts, modules, and product-specific code together.

## Dependency rules

| Layer | May depend on | Must not depend on |
|-------|---------------|-------------------|
| **Core** | Other core crates | Modules, apps |
| **Modules** | Core | Apps, other modules (avoid unless necessary) |
| **Apps** | Core, modules | — |

These rules keep the framework stable as the workspace grows from a dozen crates to fifty or more. They also make it obvious where new code belongs.

### Core

Framework contracts, hosts, and shared infrastructure. Changes here are reviewed carefully because every app and module builds on them.

Examples: `nest-core`, `nest-app`, `nest-cli`, `nest-config`, `nest-http-client`, `nest-data`.

### Modules

Adapters and integrations that wrap external systems. A module implements Nest's `Module` trait and registers services; it does not own a main loop or product UX.

Examples: `nest-airtable`, `nest-data-sqlite`. Planned: `nest-github`, `nest-postgres`, `nest-kubernetes`.

Modules should depend on core crates via `{ workspace = true }` in `Cargo.toml`, never on paths under `apps/`.

### Apps

End-user products. An app chooses a host (`nest-cli`, `nest-gui`, …), enables modules, and adds commands, views, or domain logic specific to that product.

Examples: [airtable-sync](../apps/airtable-sync/) ([pacificnm/airtable-sync](https://github.com/pacificnm/airtable-sync)), `kiwi`, `finch` (planned).

Each product folder holds its crates under `crates/` — e.g. `apps/airtable-sync/crates/core`, `apps/airtable-sync/crates/cli`, `apps/airtable-sync/crates/gui` (planned).

Apps may depend on any core crate and any module they need. Core and modules must never depend back on an app.

### App folder isolation

Everything product-specific stays inside `apps/<product>/`:

- Source crates under `apps/<product>/crates/`
- Binaries and build cache under `apps/<product>/target/` (via each app's `build` script)
- Local config, logs, and runtime files in that app folder — not the repo root

The root `target/` is for framework development (`cargo test --workspace`, core crate work). Day-to-day app work uses `./apps/<product>/build` so the root stays clean.

## Where does new code go?

| You are building… | Put it in… |
|-------------------|------------|
| Module system, service registry, lifecycle | `core/crates/nest-core` |
| New host (HTTP server, etc.) | `core/crates/nest-*` |
| Third-party API or database adapter | `modules/crates/nest-*` |
| Product-specific commands, UI, workflows | `apps/<product>/crates/` (e.g. `apps/airtable-sync/crates/core`) |
| Small demo or spike | `examples/` (not in workspace until promoted) |

When unsure: if it must ship with every Nest consumer, it is **core**. If it is optional and wraps something outside Nest, it is a **module**. If it is a product someone runs, it is an **app**.

## Workspace dependencies

Cross-layer paths are centralized in the root [`Cargo.toml`](../Cargo.toml) under `[workspace.dependencies]`. Crates reference siblings with `{ workspace = true }` so moves between layers do not require wide path churn.

## Related docs

- [README](../README.md) — crate catalog
- [nest-core](nest-core/README.md) — `AppBuilder`, modules, services
- [nest-app](nest-app/README.md) — application container
