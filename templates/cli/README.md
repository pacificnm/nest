# {{display_title}}

Nest CLI app scaffolded from [`templates/cli`](../../templates/cli) by
`scripts/scaffold-cli-app.sh`.

## Structure

```text
{{app_id}}/
├── Cargo.toml            # workspace
├── crates/
│   ├── core/             # domain logic + shared services
│   └── cli/              # thin CLI host adapter
└── build                 # standard Nest build helper
```

## Quick start

```bash
./build dev        # cargo run (debug)
./build run        # cargo run --release
./build build      # cargo build --release
./build test       # cargo test
./build check      # fmt, clippy, tests
```

Business logic lives in `crates/core`. `crates/cli` only parses arguments
and delegates to the core library.

See [Nest build standard](../../docs/build.md) and
[nest-cli docs](../../docs/nest-cli/README.md).
