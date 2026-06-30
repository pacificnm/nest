# Nest repository architecture

Nest is organized in three layers. **This repository** contains only **core** and **modules**. **Apps** are separate Git repositories.

## Layers

```text
Apps          separate repos     shipping products (airtable-sync, kiwi, …)
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

Hosts own presentation (CLI parsing, event loops, logging init). Modules register services into the Nest container. Apps wire hosts, modules, and product-specific code together — in their **own repos**, not here.

## Dependency rules

| Layer | May depend on | Must not depend on |
|-------|---------------|-------------------|
| **Core** | Other core crates | Modules, apps |
| **Modules** | Core | Apps, other modules (avoid unless necessary) |
| **Apps** | Core, modules (via git/path) | — |

### Core

Framework contracts, hosts, and shared infrastructure. Changes here are reviewed carefully because every app and module builds on them.

Examples: `nest-core`, `nest-app`, `nest-cli`, `nest-config`, `nest-http-client`, `nest-data`.

### Modules

Adapters and integrations that wrap external systems. A module implements Nest's `Module` trait and registers services; it does not own a main loop or product UX.

Examples: `nest-airtable`, `nest-data-sqlite`. Planned: `nest-github`, `nest-postgres`, `nest-kubernetes`.

Modules should depend on core crates via `{ workspace = true }` in `Cargo.toml`.

### Apps

End-user products in **separate repositories**. An app chooses a host (`nest-cli`, `nest-gui`, …), enables modules, and adds commands, views, or domain logic.

Example: [pacificnm/airtable-sync](https://github.com/pacificnm/airtable-sync), checked out in this monorepo at `apps/airtable-sync/` (git submodule). Planned: `kiwi`, `finch`.

Typical layout: `crates/core`, `crates/cli`, `crates/gui` in the product repo. Nest crates via `git` dependency on [pacificnm/nest](https://github.com/pacificnm/nest), or `path` patch when developing inside the nest monorepo submodule layout.

Apps may depend on any core crate and any module they need. Core and modules must never depend back on an app. **No product source code belongs in the nest monorepo.**

## Where does new code go?

| You are building… | Put it in… |
|-------------------|------------|
| Module system, service registry, lifecycle | `core/crates/nest-core` |
| New host (HTTP server, etc.) | `core/crates/nest-*` |
| Third-party API or database adapter | `modules/crates/nest-*` |
| Product-specific commands, UI, workflows | **Separate product repo** (see [apps/README.md](../apps/README.md)) |
| Small demo or spike | `examples/` (not in workspace until promoted) |

When unsure: if it must ship with every Nest consumer, it is **core**. If it is optional and wraps something outside Nest, it is a **module**. If it is a product someone runs, it is an **app** — in its own repository.

## Workspace dependencies

Cross-layer paths for **core** and **modules** are centralized in the root [`Cargo.toml`](../Cargo.toml) under `[workspace.dependencies]`. Crates reference siblings with `{ workspace = true }`.

The root workspace `members` list includes **only** `core/crates/*` and `modules/crates/*`.

## Related docs

- [README](../README.md) — crate catalog
- [apps/README.md](../apps/README.md) — external product repositories
- [nest-core](nest-core/README.md) — `AppBuilder`, modules, services
- [nest-app](nest-app/README.md) — application container
