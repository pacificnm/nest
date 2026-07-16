# Dependency and layering rules

Nest has three layers. Dependencies only flow one direction:

| Layer | Path | May depend on | Must not depend on |
|-------|------|---------------|---------------------|
| **Core** | `core/crates/` | Other core crates | Modules, apps |
| **Modules** | `modules/crates/` | Core | Apps, other modules (avoid unless necessary) |
| **Apps** | separate repos, local checkout `apps/<name>/` | Core, modules (via `git` or `path` patch) | — |

Core and modules must never depend back on an app. **No product source code
belongs in the nest monorepo** — apps live in their own repositories (see
[apps/README.md](../../apps/README.md)).

## Where new code goes

| Building… | Put it in… |
|-----------|------------|
| Module system, service registry, lifecycle | `core/crates/nest-core` |
| A new host (HTTP server, CLI, TUI, Tauri) | `core/crates/nest-*` |
| A third-party API or database adapter | `modules/crates/nest-*` |
| Product-specific commands, UI, workflows | Separate product repo under `apps/` |
| A small demo or spike | `examples/` (not in the workspace until promoted) |

When unsure: if it must ship with every Nest consumer, it's **core**. If it's
optional and wraps something outside Nest, it's a **module**. If it's a
product someone runs, it's an **app** — in its own repository.

Modules should depend on core crates via `{ workspace = true }` in
`Cargo.toml`. Cross-layer paths for core and modules are centralized in the
root [`Cargo.toml`](../../Cargo.toml) under `[workspace.dependencies]`.

Full detail: [docs/architecture.md](../../docs/architecture.md).
