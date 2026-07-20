# Nest App Scaffolding

Nest provides opinionated scaffolds for the app layouts used across the
framework. Each scaffold creates a runnable project with the same build
interface, the same core/adapter split, and the same dependency on Nest shared
crates.

For the runtime architecture these scaffolds implement, see
[app-standard.md](app-standard.md). For the build interface every app uses, see
[build.md](build.md). For layering additional integrations onto an already
scaffolded app, see [recipes.md](recipes.md).

## Scaffold scripts

All scaffolds live in `scripts/` and take the same basic shape:

```bash
scripts/scaffold-<type>-app.sh <target-dir> [display-title]
```

| Script | Template | What it creates |
|--------|----------|-----------------|
| [`scripts/scaffold-cli-app.sh`](../scripts/scaffold-cli-app.sh) | [`templates/cli`](../templates/cli) | Workspace with `crates/core` + `crates/cli` |
| [`scripts/scaffold-tui-app.sh`](../scripts/scaffold-tui-app.sh) | [`templates/tui`](../templates/tui) | Workspace with `crates/core` + `crates/tui` (Ratatui) |
| [`scripts/scaffold-desktop-app.sh`](../scripts/scaffold-desktop-app.sh) | [`templates/desktop`](../templates/desktop) | Tauri + React + Tailwind desktop app with a standalone CLI binary |
| [`scripts/scaffold-product-app.sh`](../scripts/scaffold-product-app.sh) | [`templates/product`](../templates/product) | Multi-surface product: desktop, TUI, and CLI sharing one `crates/core` |

Shared helper code is in [`scripts/nest-scaffold/lib.sh`](../scripts/nest-scaffold/lib.sh).

## Common behavior

Every scaffold script:

1. **Derives an app id** from the target directory name. `My App` becomes
   `my-app` (kebab-case).
2. **Derives a display title** from the app id, or uses the optional second
   argument.
3. **Refuses to overwrite** an existing directory that already contains the
   relevant files.
4. **Copies the template** into the target directory, including dotfiles.
5. **Substitutes placeholders** (`{{app_id}}`, `{{app_id_snake}}`,
   `{{display_title}}`) so crate names, package names, identifiers, and titles
   match the new app.
6. **Makes `./build` executable** so the app is runnable immediately.

## CLI app

```bash
scripts/scaffold-cli-app.sh apps/my-cli "My CLI"
cd apps/my-cli
./build dev
```

Layout:

```text
my-cli/
├── Cargo.toml            # workspace
├── crates/
│   ├── core/             # domain logic + shared services
│   └── cli/              # thin CLI host adapter
├── build                 # standard Nest build helper
└── README.md
```

Business logic lives in `crates/core`. `crates/cli` only parses arguments and
delegates to the core library.

## TUI app

```bash
scripts/scaffold-tui-app.sh apps/my-tui "My TUI"
cd apps/my-tui
./build dev
```

Layout:

```text
my-tui/
├── Cargo.toml            # workspace
├── crates/
│   ├── core/             # domain logic + shared services
│   └── tui/              # thin Ratatui host adapter
│       └── src/
│           ├── main.rs   # TuiApp setup
│           └── screens/  # TUI screens
├── build
└── README.md
```

The generated app binds `g` to a sample `greet` call into the core and `q` to
quit.

## Desktop app

```bash
scripts/scaffold-desktop-app.sh apps/my-desktop "My Desktop"
cd apps/my-desktop
./build dev
```

Layout:

```text
my-desktop/
├── build                 # standard Nest build helper
├── nest-app.toml         # Nest Shell app metadata
├── src-cli/              # standalone CLI binary (shares commands via IPC)
├── src-tauri/            # Tauri backend + commands
└── ui/                   # React + Vite + Tailwind frontend
    └── src/
        ├── App.tsx
        └── lib/nest.ts   # thin IPC helpers
```

The desktop scaffold produces two binaries:

- `src-tauri/` — the Tauri-hosted desktop app.
- `src-cli/` — a standalone CLI that exposes the same commands without the
  webview.

Command logic lives behind IPC; the React UI does not duplicate domain logic.

## Product app (multi-surface)

```bash
scripts/scaffold-product-app.sh apps/my-product "My Product"
cd apps/my-product
./build desktop dev
./build tui run
./build cli run greet World
```

Layout:

```text
my-product/
├── Cargo.toml            # workspace
├── config.toml           # shared app config
├── crates/
│   └── core/             # single shared core for all surfaces
├── cli/                  # CLI surface
├── tui/                  # Ratatui surface
├── desktop/              # Tauri + React surface
│   ├── src-tauri/
│   └── ui/
├── build                 # root dispatcher
└── README.md
```

The root `./build` script dispatches to each surface:

```bash
./build desktop dev
./build tui run
./build cli build
./build all check
```

Omitting the surface defaults to `desktop`; omitting the command defaults to
`build`.

## Shared scaffold library

[`scripts/nest-scaffold/lib.sh`](../scripts/nest-scaffold/lib.sh) provides the
common helpers used by every scaffold script:

| Function | Purpose |
|----------|---------|
| `nest_scaffold_find_root` | Locate the Nest repository root from the script location |
| `nest_scaffold_parse_args` | Parse `<target-dir> [display-title]` |
| `nest_scaffold_resolve_names` | Resolve `APP_NAME`, `APP_ID`, and `APP_TITLE` |
| `nest_scaffold_derive_app_id` | Convert a directory name to kebab-case |
| `nest_scaffold_derive_app_title` | Convert kebab-case to title case |
| `nest_scaffold_derive_app_id_snake` | Convert kebab-case to snake_case |
| `nest_scaffold_replace_mustache` | Replace `{{app_id}}`, `{{app_id_snake}}`, `{{display_title}}` in a file |

## Placeholders

Templates use three mustache-style placeholders:

| Placeholder | Example | Usage |
|-------------|---------|-------|
| `{{app_id}}` | `my-app` | Crate names, Tauri bundle id, file paths |
| `{{app_id_snake}}` | `my_app` | Rust crate name for the core library, module paths |
| `{{display_title}}` | `My App` | Human-readable titles in READMEs, UI labels |

## After scaffolding

1. `cd` into the new app directory.
2. Run `./build dev` or `./build run`.
3. Open `crates/core/src/lib.rs` (or `src-tauri/src/main.rs` / `src-cli/src/main.rs`
   for desktop) and start adding domain logic.
4. Use recipes to add optional integrations:
   ```bash
   scripts/recipes/database-sqlite.sh apps/my-product
   ```

## Adding or changing a template

1. Edit the template directory under `templates/`.
2. Use the placeholders above so scaffold scripts can substitute names.
3. Keep the `./build` helper executable.
4. Update this document and any relevant README files.

If you change a template, existing scaffolded apps are not regenerated; recipes
are the supported way to evolve already-created apps.
