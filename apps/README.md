# Nest applications

Product apps are **separate Git repositories**, cloned into `apps/<product>/` as **git submodules**. They are not part of the Nest framework workspace (`core/` + `modules/` only).

## Setup

After cloning nest:

```bash
git submodule update --init apps/airtable-sync
```

Or clone with submodules:

```bash
git clone --recurse-submodules https://github.com/pacificnm/nest.git
```

## Products

| Path | Repository |
|------|------------|
| `apps/airtable-sync/` | [github.com/pacificnm/airtable-sync](https://github.com/pacificnm/airtable-sync) |

Planned: `kiwi`, `finch`, …

## Build airtable-sync

From the app directory (uses local Nest via `.cargo/config.toml` patch when inside this monorepo):

```bash
cd apps/airtable-sync
cp config.example.toml config.toml
export AIRTABLE_TOKEN="pat..."
./build build
./build run -- tables
```

Or from repo root:

```bash
./apps/airtable-sync/build run -- tables
```

## Adding another app

```bash
git submodule add https://github.com/pacificnm/<product>.git apps/<product>
```

**Dependency rule:** products depend on Nest core and modules only. See [docs/architecture.md](../docs/architecture.md).
