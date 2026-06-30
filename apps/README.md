# Nest applications

Each product is **self-contained** under `apps/<product>/`. The repo root and `core/` stay framework-only — no app binaries, logs, or local config at the root.

```text
apps/
└── airtable-sync/
    ├── README.md
    ├── build                 # build/run helper (local target/)
    ├── config.example.toml
    ├── config.toml           # local only (gitignored)
    ├── logs/                 # local only (gitignored)
    ├── target/               # binaries + build cache (gitignored)
    └── crates/
        ├── core/
        ├── cli/
        └── gui/              # planned
```

## What lives in the app folder

| Item | Location |
|------|----------|
| Product crates | `crates/core`, `crates/cli`, `crates/gui`, … |
| Release/debug binaries | `target/debug/`, `target/release/` |
| App config | `config.toml` (copy from `config.example.toml`) |
| Log files | `logs/` (when `[logging]` uses `./logs`) |
| Product README & examples | app root |

## What stays out of the app folder

| Item | Location |
|------|----------|
| Framework crates | `core/crates/` |
| Shared integrations | `modules/crates/` |
| Framework docs | `docs/` |
| Workspace `Cargo.toml` | repo root (lists members only) |

## Building an app

Always use the app's `build` script so artifacts land under that app's `target/`:

```bash
./apps/airtable-sync/build build
./apps/airtable-sync/build release
./apps/airtable-sync/build run -- tables
```

`run` executes from the app directory and picks up `config.toml` there automatically.

Avoid `cargo build -p airtable-sync-cli` from the repo root unless you intentionally want the shared root `target/`.

## Adding a new app

1. Create `apps/<name>/crates/{core,cli,...}`
2. Add `build`, `config.example.toml`, `.gitignore` (`target/`, `logs/`, `config.toml`)
3. Register crate paths in root [`Cargo.toml`](../Cargo.toml) `members`

**Dependency rule:** apps depend on `core/` and `modules/` only. See [docs/architecture.md](../docs/architecture.md).
