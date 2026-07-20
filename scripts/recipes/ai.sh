#!/usr/bin/env bash
# Recipe: ai
# Layers nest-ai + Ollama and Claude providers onto an existing Nest app.
#
# Usage:
#   scripts/recipes/ai.sh <app-dir>
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

RECIPE_ID="ai"
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

# Add AI crates as path dependencies to the core crate.
# OllamaModule depends on the shared HTTP client module, so add it explicitly.
recipe_add_path_dep "$CORE_CARGO" "nest-ai" "core/crates/nest-ai"
recipe_add_path_dep "$CORE_CARGO" "nest-ai-ollama" "modules/crates/nest-ai-ollama"
recipe_add_path_dep "$CORE_CARGO" "nest-claude" "modules/crates/nest-claude"
recipe_add_path_dep "$CORE_CARGO" "nest-http-client" "core/crates/nest-http-client"

# Write a wiring example file. Rename to ai.rs and add `pub mod ai;` to
# crates/core/src/lib.rs, then wire the modules into each surface host.
cat > "$CORE_SRC/ai.rs.example" <<EOF
//! Example AI wiring for $APP_ID.
//!
//! Wire this into each surface host (cli/src/main.rs, tui/src/main.rs,
//! desktop/src-tauri/src/main.rs):
//!
//! \`\`\`ignore
//! use nest_http_client::HttpClientModule;
//! use nest_ai_ollama::{OllamaConfig, OllamaModule};
//! use nest_claude::{ClaudeConfig, ClaudeModule};
//! use ${APP_ID_SNAKE}_core::ai::${APP_ID_PASCAL}AiModule;
//!
//! fn main() -> nest_error::NestResult<()> {
//!     CliApp::new("...")
//!         .module(HttpClientModule::default())
//!         .module(OllamaModule::with_config(OllamaConfig::default_local()))
//!         .module(ClaudeModule::with_config(ClaudeConfig::from_env()?))
//!         .module(${APP_ID_PASCAL}AiModule)
//!         .run();
//!     Ok(())
//! }
//! \`\`\`
//!
//! Use the AI service in core code:
//!
//! \`\`\`ignore
//! use nest_ai::{AiProvider, CompletionRequest};
//! use nest_ai_ollama::OllamaProvider;
//!
//! let ai: std::sync::Arc<dyn AiProvider> = std::sync::Arc::new(OllamaProvider::new(config)?);
//! let response = ai.complete(CompletionRequest::user_message("Hello")).await?;
//! \`\`\`

use nest_core::{AppBuilder, Module, ModuleId, NestResult};
use nest_ai_ollama::OLLAMA_MODULE_ID;
use nest_claude::CLAUDE_MODULE_ID;

/// App-specific AI module. Depends on the Ollama and Claude providers.
pub struct ${APP_ID_PASCAL}AiModule;

impl Module for ${APP_ID_PASCAL}AiModule {
    fn id(&self) -> ModuleId {
        ModuleId("$APP_ID-ai")
    }

    fn dependencies(&self) -> &'static [ModuleId] {
        &[OLLAMA_MODULE_ID, CLAUDE_MODULE_ID]
    }

    fn configure(&self, _app: &mut AppBuilder) -> NestResult<()> {
        // Register app-specific AI clients or prompt templates here.
        Ok(())
    }
}
EOF

# Add a commented config section if config.toml exists and doesn't already have [ai].
CONFIG_FILE="$APP_DIR/config.toml"
if [[ -f "$CONFIG_FILE" ]] && ! grep -q "^\[ai\]" "$CONFIG_FILE"; then
  cat >> "$CONFIG_FILE" <<EOF

[ai]
# enabled = true
# provider = "ollama"
# base_url = "http://127.0.0.1:11434"
# model = "smollm2:360m"

[claude]
# api_key_env = "ANTHROPIC_API_KEY"
# default_model = "claude-opus-4-8"
# default_max_tokens = 4096
EOF
fi

recipe_record_applied "$APP_DIR" "$RECIPE_ID" "$RECIPE_VERSION"

echo
echo "Recipe '$RECIPE_ID' applied successfully."
echo "Next steps:"
echo "  1. Review $CORE_SRC/ai.rs.example"
echo "  2. Rename it to ai.rs and add pub mod ai to crates/core/src/lib.rs"
echo "  3. Wire HttpClientModule, OllamaModule, ClaudeModule, and ${APP_ID_PASCAL}AiModule into your surface hosts"
echo "  4. Ensure ANTHROPIC_API_KEY is exported for Claude, or Ollama is running locally"
echo "  5. Uncomment/adjust [ai] and [claude] in $CONFIG_FILE"
echo "  6. Delete $CORE_SRC/ai.rs.example once wired"
