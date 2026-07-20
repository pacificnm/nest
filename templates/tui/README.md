# {{display_title}}

Nest TUI app (Ratatui), scaffolded from [`templates/tui`](../../templates/tui) by
`scripts/scaffold-tui-app.sh`.

## Structure

```text
{{app_id}}/
├── Cargo.toml            # workspace
├── crates/
│   ├── core/             # domain logic + shared services
│   └── tui/              # thin Ratatui host adapter
│       └── src/
│           ├── main.rs   # TuiApp setup
│           └── screens/  # TUI screens
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

Business logic lives in `crates/core`. `crates/tui` only handles the terminal
UI and delegates to the core library.

Press `g` in the app to call `{{app_id_snake}}_core::greet`, `q` to quit.

See [Nest build standard](../../docs/build.md) and
[nest-tui docs](../../docs/nest-tui/README.md).
