# Nest App Recipes

Recipes are small, idempotent shell scripts that layer an optional integration
onto an already-scaffolded Nest app. They live in
[`scripts/recipes/`](../scripts/recipes/) and are cataloged in
[`scripts/recipes/registry.json`](../scripts/recipes/registry.json).

For the broader app layout (Rust core, hosts, IPC, folder structure), see
[app-standard.md](app-standard.md). For how to create a new app, see
[build.md](build.md) and the scaffold scripts referenced there.

## When to use a recipe

Use a recipe when you have an existing app and want to add a capability without
regenerating the project:

- Add a SQLite data layer.
- Add a PostgreSQL data layer.
- Future recipes may add caching, HTTP clients, task scheduling, AI providers,
  etc.

Recipes only modify the app they are pointed at; they never change templates or
shared crates.

## Available recipes

| Recipe | Command | Adds |
|--------|---------|------|
| SQLite database | `scripts/recipes/database-sqlite.sh <app-dir>` | `nest-data`, `nest-data-sqlite` |
| PostgreSQL database | `scripts/recipes/database-postgres.sh <app-dir>` | `nest-data` (async), `nest-data-postgres` |
| HTTP client | `scripts/recipes/http-client.sh <app-dir>` | `nest-http-client` |
| AI providers | `scripts/recipes/ai.sh <app-dir>` | `nest-ai`, `nest-ai-ollama`, `nest-claude`, `nest-http-client` |

## How recipes work

Every recipe follows the same steps:

1. **Locate the app.** The argument must be a directory containing a
   `Cargo.toml` workspace.
2. **Locate the core crate.** Recipes assume the product / CLI / TUI layout
   where the shared core crate lives at `crates/core/Cargo.toml`.
3. **Check idempotency.** Each recipe records its id in `<app-dir>/.nest-recipes`
   and refuses to re-apply if already present.
4. **Add dependencies.** Recipes use `cargo add --path` to wire Nest crates into
   `crates/core/Cargo.toml`.
5. **Write a wiring example.** An `.rs.example` file is created under
   `crates/core/src/` showing how to register the provider module and define an
   app-specific module that depends on it.
6. **Append config.** If `<app-dir>/config.toml` exists and does not already
   contain a relevant section, commented-out config is appended.
7. **Record completion.** The recipe id and version are recorded in
   `.nest-recipes`.

## Recipe tracking and versions

`.nest-recipes` is a line-oriented file in the app root:

```text
database-sqlite@1.0.0
database-postgres@1.0.0
http-client@1.0.0
ai@1.0.0
```

Each line is `id@version`. Re-applying a recipe updates its version entry.
Product app templates include a `ListRecipes` shared command that reads this
file and prints the loaded recipes, so it can be called from the CLI or surfaced
in the desktop UI (for example, under **Developer → Show loaded recipes**).

## Applying a recipe

Run a recipe from the Nest repository root, passing the app directory:

```bash
# SQLite
scripts/recipes/database-sqlite.sh apps/my-app

# PostgreSQL
scripts/recipes/database-postgres.sh apps/my-app
```

Both can be applied to the same app if you need both providers. Re-running a
recipe exits cleanly:

```bash
scripts/recipes/database-sqlite.sh apps/my-app
# Recipe 'database-sqlite' is already applied to /home/jaimie/projects/apps/my-app
```

## After applying a database recipe

The recipe adds the crates but does not wire them into surface hosts
automatically. Follow the printed next steps:

1. Review the generated example file:
   - SQLite: `crates/core/src/data_sqlite.rs.example`
   - PostgreSQL: `crates/core/src/data_postgres.rs.example`
2. Rename it to a real Rust module:
   - `mv crates/core/src/data_sqlite.rs.example crates/core/src/data_sqlite.rs`
   - `mv crates/core/src/data_postgres.rs.example crates/core/src/data_postgres.rs`
3. Export the module from `crates/core/src/lib.rs`:
   ```rust
   pub mod data_sqlite;      // or data_postgres
   ```
4. Wire the provider and app-specific module into each surface host
   (`cli/src/main.rs`, `tui/src/main.rs`, `desktop/src-tauri/src/main.rs`):
   ```rust
   use nest_data::DataModule;
   use nest_data_sqlite::SqliteDataModule;
   use my_app_core::data_sqlite::MyAppDataModule;

   CliApp::new("...")
       .module(DataModule)
       .module(SqliteDataModule::primary("my-app.db"))
       .module(MyAppDataModule)
       .run();
   ```

   For the HTTP-client recipe the pattern is similar:
   ```rust
   use nest_http_client::HttpClientModule;
   use my_app_core::http_client::MyAppHttpModule;

   CliApp::new("...")
       .module(HttpClientModule::default())
       .module(MyAppHttpModule)
       .run();
   ```

   For the AI recipe, wire the shared HTTP client plus both providers:
   ```rust
   use nest_http_client::HttpClientModule;
   use nest_ai_ollama::{OllamaConfig, OllamaModule};
   use nest_claude::{ClaudeConfig, ClaudeModule};
   use my_app_core::ai::MyAppAiModule;

   fn main() -> nest_error::NestResult<()> {
       CliApp::new("...")
           .module(HttpClientModule::default())
           .module(OllamaModule::with_config(OllamaConfig::default_local()))
           .module(ClaudeModule::with_config(ClaudeConfig::from_env()?))
           .module(MyAppAiModule)
           .run();
       Ok(())
   }
   ```
5. Uncomment and adjust the `[database]` section in `config.toml` if you use it.
6. Delete the `.example` file once the code is wired.

## Recipe registry

[`scripts/recipes/registry.json`](../scripts/recipes/registry.json) lists every
recipe with its id, display name, description, script path, and the app layouts
it supports:

```json
{
  "recipes": [
    {
      "id": "database-sqlite",
      "name": "SQLite database",
      "description": "Local SQLite data layer via nest-data-sqlite",
      "script": "scripts/recipes/database-sqlite.sh",
      "applies_to": ["product", "cli", "tui"]
    },
    {
      "id": "database-postgres",
      "name": "PostgreSQL database",
      "description": "Async PostgreSQL + pgvector via nest-data-postgres",
      "script": "scripts/recipes/database-postgres.sh",
      "applies_to": ["product", "cli", "tui"]
    },
    {
      "id": "http-client",
      "name": "HTTP client",
      "description": "Consume HTTP/HTTPS APIs via nest-http-client",
      "script": "scripts/recipes/http-client.sh",
      "applies_to": ["product", "cli", "tui"]
    },
    {
      "id": "ai",
      "name": "AI providers",
      "description": "AI inference via nest-ai, Ollama, and Claude",
      "script": "scripts/recipes/ai.sh",
      "applies_to": ["product", "cli", "tui"]
    }
  ]
}
```

The registry is intended for future tooling (for example, a `nest recipe list`
command or IDE integration) but is also useful reference for humans.

## Adding a new recipe

1. Create `scripts/recipes/<name>.sh` and make it executable.
2. Source `scripts/recipes/lib.sh` and call its helpers for idempotency,
   dependency injection, and naming.
3. Set `RECIPE_VERSION="1.0.0"` and pass it to `recipe_record_applied`.
4. Add an entry to `scripts/recipes/registry.json`.
5. Update this document.

Keep recipes minimal: add dependencies, generate an example file, append config,
and record completion. Avoid modifying surface host code automatically; that is
best left to the developer.

## Recipe internals

- `RECIPE_NEST_ROOT` is resolved from the recipe script location, so recipes can
  find Nest crates by relative path regardless of the current working
  directory.
- App ids are derived from the directory name using the same kebab-case rules
  as the scaffold scripts, and converted to `snake_case` and `PascalCase` for
  crate and module identifiers.
- Dependencies are added with `cargo add --path`, which keeps the generated
  `Cargo.toml` valid even if the app is moved inside the Nest repo.
