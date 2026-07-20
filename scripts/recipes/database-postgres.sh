#!/usr/bin/env bash
# Recipe: database-postgres
# Layers nest-data (async) + nest-data-postgres onto an existing Nest app.
#
# Usage:
#   scripts/recipes/database-postgres.sh <app-dir>
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# shellcheck source=/dev/null
source "$SCRIPT_DIR/lib.sh"

recipe_find_nest_root "$SCRIPT_DIR"

if [[ $# -ne 1 ]]; then
  echo "Usage: ${0##*/} <app-dir>" >&2
  exit 1
fi

APP_DIR="$(recipe_resolve_app_dir "$1")"
recipe_ensure_app_root "$APP_DIR"

RECIPE_ID="database-postgres"
RECIPE_VERSION="1.0.0"
if recipe_is_applied "$APP_DIR" "$RECIPE_ID"; then
  echo "Recipe '$RECIPE_ID' is already applied to $APP_DIR"
  exit 0
fi

echo "Applying recipe: $RECIPE_ID"
echo "  app dir: $APP_DIR"

CORE_CARGO="$(recipe_core_cargo_toml "$APP_DIR")"
CORE_SRC="$(recipe_core_src_dir "$APP_DIR")"
APP_ID="$(recipe_derive_app_id "$(basename "$APP_DIR")")"
APP_ID_SNAKE="$(recipe_derive_app_id_snake "$APP_ID")"
APP_ID_PASCAL="$(recipe_derive_app_id_pascal "$APP_ID")"

echo "  app id: $APP_ID"
echo "  core crate: $CORE_CARGO"

# Add data crates as path dependencies to the core crate.
# Postgres needs nest-data's async feature enabled.
recipe_add_path_dep "$CORE_CARGO" "nest-data" "core/crates/nest-data" "async"
recipe_add_path_dep "$CORE_CARGO" "nest-data-postgres" "modules/crates/nest-data-postgres"

# Write a wiring example file. Rename to data_postgres.rs and add
# `pub mod data_postgres;` to crates/core/src/lib.rs, then wire the modules into
# each surface host.
cat > "$CORE_SRC/data_postgres.rs.example" <<EOF
//! Example PostgreSQL data-layer wiring for $APP_ID.
//!
//! Wire this into each surface host (cli/src/main.rs, tui/src/main.rs,
//! desktop/src-tauri/src/main.rs):
//!
//! \`\`\`ignore
//! use nest_data::DataModule;
//! use nest_data_postgres::PostgresDataModule;
//! use ${APP_ID_SNAKE}_core::data_postgres::${APP_ID_PASCAL}DataModule;
//!
//! CliApp::new("...")
//!     .module(DataModule)
//!     .module(PostgresDataModule::from_env("DATABASE_URL")?)
//!     .module(${APP_ID_PASCAL}DataModule)
//!     .run();
//! \`\`\`

use nest_core::{AppBuilder, Module, ModuleId, NestResult};
use nest_data_postgres::POSTGRES_DATA_MODULE_ID;

/// App-specific data module. Depends on the PostgreSQL provider.
pub struct ${APP_ID_PASCAL}DataModule;

impl Module for ${APP_ID_PASCAL}DataModule {
    fn id(&self) -> ModuleId {
        ModuleId("$APP_ID-data")
    }

    fn dependencies(&self) -> &'static [ModuleId] {
        &[POSTGRES_DATA_MODULE_ID]
    }

    fn configure(&self, _app: &mut AppBuilder) -> NestResult<()> {
        // Wire app-specific repositories here.
        Ok(())
    }
}
EOF

# Add a commented config section if config.toml exists and doesn't already have [database].
CONFIG_FILE="$APP_DIR/config.toml"
if [[ -f "$CONFIG_FILE" ]] && ! grep -q "^\[database\]" "$CONFIG_FILE"; then
  cat >> "$CONFIG_FILE" <<EOF

[database]
# PostgreSQL connection URL. The environment variable DATABASE_URL takes precedence
# when using PostgresDataModule::from_env.
# url = "postgresql://${APP_ID_SNAKE}:CHANGE_ME@server.lan:5432/${APP_ID_SNAKE}"
EOF
fi

recipe_record_applied "$APP_DIR" "$RECIPE_ID" "$RECIPE_VERSION"

echo
echo "Recipe '$RECIPE_ID' applied successfully."
echo "Next steps:"
echo "  1. Review $CORE_SRC/data_postgres.rs.example"
echo "  2. Rename it to data_postgres.rs and add pub mod data_postgres to crates/core/src/lib.rs"
echo "  3. Wire DataModule, PostgresDataModule, and ${APP_ID_PASCAL}DataModule into your surface hosts"
echo "  4. Uncomment/adjust [database] in $CONFIG_FILE"
echo "  5. Ensure PostgreSQL + pgvector are available (see docs/nest-data-postgres/README.md)"
echo "  6. Delete $CORE_SRC/data_postgres.rs.example once wired"
