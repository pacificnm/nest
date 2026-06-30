# Nest applications

**Applications do not live in this repository.**

The Nest monorepo contains only the **framework** (`core/crates/`) and **integration modules** (`modules/crates/`). Shipping products are separate Git repositories that depend on [pacificnm/nest](https://github.com/pacificnm/nest).

## Products

| Product | Repository |
|---------|------------|
| **airtable-sync** | [github.com/pacificnm/airtable-sync](https://github.com/pacificnm/airtable-sync) |

Planned: `kiwi`, `finch`, …

## Typical product layout

```text
<product-repo>/
├── Cargo.toml              # workspace; nest crates via git (or path patch locally)
├── build
├── config.example.toml
├── target/                 # gitignored
└── crates/
    ├── core/
    ├── cli/
    └── gui/                # optional
```

## Dependency rule

Products depend on Nest **core** and **modules** only. Nothing in `core/` or `modules/` may depend on a product.

See [docs/architecture.md](../docs/architecture.md).

## Local development with a sibling Nest checkout

In the product repo, add `.cargo/config.toml`:

```toml
[patch."https://github.com/pacificnm/nest.git"]
nest-cli = { path = "../nest/core/crates/nest-cli" }
nest-airtable = { path = "../nest/modules/crates/nest-airtable" }
# … other nest crates as needed
```
