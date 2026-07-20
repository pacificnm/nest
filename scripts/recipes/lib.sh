#!/usr/bin/env bash
# Shared helpers for Nest recipe scripts.
#
# Recipes layer optional integrations onto an already-scaffolded app. They are
# idempotent: each recipe records its id in <app-root>/.nest-recipes and refuses
# to re-apply.

set -euo pipefail

# Find the Nest repository root by walking up from the recipe script.
# Sets RECIPE_NEST_ROOT. Exits if not found.
recipe_find_nest_root() {
  local script_dir="$1"
  RECIPE_NEST_ROOT="$(cd "$script_dir/../.." && pwd)"
  if [[ ! -f "$RECIPE_NEST_ROOT/scripts/nest-build/lib.sh" ]]; then
    echo "error: cannot locate Nest repository root from $script_dir" >&2
    exit 1
  fi
}

# Resolve an app directory to an absolute path.
recipe_resolve_app_dir() {
  local app_dir="$1"
  if [[ "$app_dir" != /* ]]; then
    app_dir="$(pwd)/$app_dir"
  fi
  echo "$app_dir"
}

# Print an error and exit if the app directory does not look like a Nest app.
recipe_ensure_app_root() {
  local app_dir="$1"
  if [[ ! -f "$app_dir/Cargo.toml" ]]; then
    echo "error: $app_dir does not contain a Cargo.toml workspace" >&2
    exit 1
  fi
}

# Derive a kebab-case app id from a directory name.
recipe_derive_app_id() {
  local name="$1"
  echo "$name" | tr '[:upper:]' '[:lower:]' | sed -E 's/[^a-z0-9]+/-/g; s/^-+//; s/-+$//'
}

# Convert a kebab-case id to a snake_case Rust identifier.
recipe_derive_app_id_snake() {
  local app_id="$1"
  echo "$app_id" | sed -E 's/-/_/g'
}

# Convert a kebab-case id to a PascalCase Rust identifier.
recipe_derive_app_id_pascal() {
  local app_id="$1"
  echo "$app_id" | sed -E 's/(^|-)([a-z])/\U\2/g; s/-//g'
}

# Locate the core crate Cargo.toml. Supports product/cli/tui layouts where the
# core crate lives at crates/core/Cargo.toml.
recipe_core_cargo_toml() {
  local app_dir="$1"
  echo "$app_dir/crates/core/Cargo.toml"
}

# Locate the core crate src directory.
recipe_core_src_dir() {
  local app_dir="$1"
  echo "$app_dir/crates/core/src"
}

# Check whether a recipe has already been applied.
# Recipes are tracked in <app-dir>/.nest-recipes as "id@version" lines.
recipe_is_applied() {
  local app_dir="$1"
  local recipe_id="$2"
  [[ -f "$app_dir/.nest-recipes" ]] && grep -qE "^${recipe_id}@" "$app_dir/.nest-recipes"
}

# Record a recipe as applied with a version.
# Usage: recipe_record_applied <app-dir> <recipe-id> <version>
recipe_record_applied() {
  local app_dir="$1"
  local recipe_id="$2"
  local version="$3"
  local recipes_file="$app_dir/.nest-recipes"

  touch "$recipes_file"

  if grep -qE "^${recipe_id}@" "$recipes_file"; then
    # Update the existing entry so re-applying records the latest version.
    sed -i -E "s/^${recipe_id}@.*/${recipe_id}@${version}/" "$recipes_file"
  else
    echo "${recipe_id}@${version}" >> "$recipes_file"
  fi
}

# Add a path dependency to a package's Cargo.toml if it is not already present.
# Usage: recipe_add_path_dep <package-manifest> <crate-name> <path-from-nest-root> [features]
recipe_add_path_dep() {
  local package_manifest="$1"
  local crate_name="$2"
  local crate_path="$3"
  local features="${4:-}"

  if [[ -z "$features" ]] && grep -q "^${crate_name} =" "$package_manifest"; then
    echo "recipe: dependency ${crate_name} already present in $(basename "$(dirname "$package_manifest")")"
    return 0
  fi

  local args=(
    --manifest-path "$package_manifest"
    --path "$RECIPE_NEST_ROOT/$crate_path"
  )
  if [[ -n "$features" ]]; then
    args+=(--features "$features")
  fi

  cargo add "${args[@]}" "$crate_name"
}
