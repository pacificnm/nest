#!/usr/bin/env bash
# Recipe: http-client
# Layers nest-http-client onto an existing Nest app so it can consume HTTP/HTTPS
# APIs.
#
# Usage:
#   scripts/recipes/http-client.sh <app-dir>
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

RECIPE_ID="http-client"
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

# Add the HTTP client crate as a path dependency to the core crate.
recipe_add_path_dep "$CORE_CARGO" "nest-http-client" "core/crates/nest-http-client"

# Write a wiring example file. Rename to http_client.rs and add
# `pub mod http_client;` to crates/core/src/lib.rs, then wire the module into
# each surface host.
cat > "$CORE_SRC/http_client.rs.example" <<EOF
//! Example HTTP-client wiring for $APP_ID.
//!
//! Wire this into each surface host (cli/src/main.rs, tui/src/main.rs,
//! desktop/src-tauri/src/main.rs):
//!
//! \`\`\`ignore
//! use nest_http_client::HttpClientModule;
//! use ${APP_ID_SNAKE}_core::http_client::${APP_ID_PASCAL}HttpModule;
//!
//! CliApp::new("...")
//!     .module(HttpClientModule::default())
//!     .module(${APP_ID_PASCAL}HttpModule)
//!     .run();
//! \`\`\`
//!
//! Use the service in core code or a surface host:
//!
//! \`\`\`ignore
//! use nest_http_client::HttpClientService;
//!
//! let http = app_context.service::<HttpClientService>()?;
//! let data: serde_json::Value = http.get_json("https://api.example.com/v1/data").await?;
//! \`\`\`

use nest_core::{AppBuilder, Module, ModuleId, NestResult};
use nest_http_client::HTTP_CLIENT_MODULE_ID;

/// App-specific HTTP client module. Depends on the shared HTTP client provider.
pub struct ${APP_ID_PASCAL}HttpModule;

impl Module for ${APP_ID_PASCAL}HttpModule {
    fn id(&self) -> ModuleId {
        ModuleId("$APP_ID-http")
    }

    fn dependencies(&self) -> &'static [ModuleId] {
        &[HTTP_CLIENT_MODULE_ID]
    }

    fn configure(&self, _app: &mut AppBuilder) -> NestResult<()> {
        // Register app-specific API clients here.
        Ok(())
    }
}
EOF

recipe_record_applied "$APP_DIR" "$RECIPE_ID" "$RECIPE_VERSION"

echo
echo "Recipe '$RECIPE_ID' applied successfully."
echo "Next steps:"
echo "  1. Review $CORE_SRC/http_client.rs.example"
echo "  2. Rename it to http_client.rs and add pub mod http_client to crates/core/src/lib.rs"
echo "  3. Wire HttpClientModule and ${APP_ID_PASCAL}HttpModule into your surface hosts"
echo "  4. Delete $CORE_SRC/http_client.rs.example once wired"
