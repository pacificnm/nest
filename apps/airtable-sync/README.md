# Airtable Sync

Sync tooling for [Airtable](https://airtable.com), built on the [Nest](https://github.com/pacificnm/nest) framework.

**Product repository:** [github.com/pacificnm/airtable-sync](https://github.com/pacificnm/airtable-sync)

## Crates

| Crate | Path | Role |
|-------|------|------|
| `airtable-sync-core` | `crates/core/` | Shared modules, commands, and sync logic |
| `airtable-sync-cli` | `crates/cli/` | CLI binary (`airtable-sync`) |

Planned: `crates/gui/` (`airtable-sync-gui`).

## Build

Use the app `build` script so binaries, cache, config, and logs stay **inside** `apps/airtable-sync/` (root stays clean):

```bash
cp config.example.toml config.toml   # from apps/airtable-sync/
export AIRTABLE_TOKEN="pat..."

../../apps/airtable-sync/build build
../../apps/airtable-sync/build release
./build run -- tables              # when cwd is apps/airtable-sync/
```

From repo root:

```bash
./apps/airtable-sync/build run -- list assets
```

- Binary: `target/release/airtable-sync` (relative to this app folder)
- Config: `config.toml` here (auto-loaded on `run`)
- Logs: `./logs/` (see `config.example.toml`)

## Quick start

```bash
cp config.example.toml config.toml
export AIRTABLE_TOKEN="pat..."
./build run -- tables
./build run -- list assets --json
```

## Configuration

Copy `config.example.toml` to `config.toml` in this folder (recommended). `run` loads it automatically.

```toml
[airtable]
base_id = "appXXXXXXXXXXXXXX"
token_env = "AIRTABLE_TOKEN"

[airtable.tables.assets]
table_id = "tblXXXXXXXXXXXXXX"
primary_key_field = "Asset ID"

[logging]
level = "info"
directory = "./logs"
```

## Commands

| Command | Description |
|---------|-------------|
| `tables` | List configured logical table names |
| `list <table>` | Fetch all records from a table (`--json` for output) |

## Layering

This app lives under `apps/airtable-sync/` and depends on Nest **core** hosts and **modules** (`nest-airtable`). See [Nest architecture](../../docs/architecture.md).
