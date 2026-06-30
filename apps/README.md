# Nest applications

**Applications do not live in this repository.** There is no `apps/<product>/` checkout in the Nest framework tree.

Products are **separate Git repositories** that depend on [pacificnm/nest](https://github.com/pacificnm/nest) via `git` (or local `path` patch). Clone them wherever you like — typically as a **sibling** of your nest checkout:

```text
~/projects/
├── nest/                 # this repo — framework only
└── airtable-sync/        # product repo
```

## Pacific NM products

| Product | Repository |
|---------|------------|
| **airtable-sync** | [github.com/pacificnm/airtable-sync](https://github.com/pacificnm/airtable-sync) |

Planned: `kiwi`, `finch`, …

## Local development (nest + product side by side)

Clone both repos, then in the **product** repo add `.cargo/config.toml`:

```toml
[patch."https://github.com/pacificnm/nest.git"]
nest-cli = { path = "../nest/core/crates/nest-cli" }
nest-airtable = { path = "../nest/modules/crates/nest-airtable" }
# … other nest crates as needed
```

Adjust `../nest` if your layout differs.

## Build airtable-sync

```bash
git clone https://github.com/pacificnm/airtable-sync.git
cd airtable-sync
cp config.example.toml config.toml
export AIRTABLE_TOKEN="pat..."
./build build
./build run -- tables
```

**Dependency rule:** products depend on Nest core and modules only. See [docs/architecture.md](../docs/architecture.md).
