# Nest CLI app template

Starter layout for a Nest **CLI** application. It provides a minimal Cargo project that imports the core Nest crates you typically need for a command‑line tool (e.g., `nest-config`, `nest-logging`, `nest-error`, `nest-task`).

## Structure

```
templates/cli/
├── src/
│   └── main.rs          # entry point with a basic Nest `run()` function
│   └── cli_command.rs   # shared command enum used by desktop & CLI
├── Cargo.toml            # workspace crate definition
├── build                 # helper script mirroring the desktop `build`
└── README.md            # this file
```

## Features

- Uses the `nest-app` crate as the entry point (provides `run()` and argument parsing).
- Sets up logging and error handling out of the box.
- Includes a sample `Config` struct loaded from `nest-config`.
- Ready to add sub‑commands via `nest-task` or any other Nest modules.

## Quick start

```bash
# Scaffold a new CLI app called "my-cli"
./scripts/scaffold-cli-app.sh apps/my-cli "My CLI"

cd apps/my-cli
./build dev        # run in debug mode (cargo run)
./build run        # run in release mode (cargo run --release)
./build build      # build the release binary only (cargo build --release)
```

The scaffold script copies this template into the target directory, rewrites the crate name, binary name, and updates the `Cargo.toml` and `README.md` with the provided display title.
