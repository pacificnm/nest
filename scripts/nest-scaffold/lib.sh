#!/usr/bin/env bash
# Nest scaffold helper library — source from scripts/scaffold-*.sh.
#
# Provides argument parsing, name derivation, and placeholder substitution
# shared across Nest app scaffold scripts.

set -euo pipefail

# Print standard scaffold usage for scripts taking <target-dir> [display-title].
# Usage: nest_scaffold_usage "$0"
nest_scaffold_usage() {
  local prog="$1"
  cat <<EOF
Usage: ${prog##*/} <target-dir> [display-title]
EOF
}

# Find the Nest repository root by walking up from the script directory.
# Sets NEST_ROOT. Exits if not found.
nest_scaffold_find_root() {
  local script_dir="$1"
  NEST_ROOT="$(cd "$script_dir/.." && pwd)"
  if [[ ! -f "$NEST_ROOT/scripts/nest-build/lib.sh" ]]; then
    echo "error: cannot locate Nest repository root from $script_dir" >&2
    exit 1
  fi
}

# Validate argument count and resolve TARGET_DIR to an absolute path.
# Sets TARGET_DIR and optional TARGET_TITLE.
nest_scaffold_parse_args() {
  if [[ $# -lt 1 || $# -gt 2 ]]; then
    nest_scaffold_usage "${0:-scaffold-app.sh}"
    exit 1
  fi

  TARGET_DIR="$1"
  if [[ "$TARGET_DIR" != /* ]]; then
    TARGET_DIR="$(pwd)/$TARGET_DIR"
  fi

  if [[ -n "${2:-}" ]]; then
    TARGET_TITLE="$2"
  else
    TARGET_TITLE=""
  fi
}

# Derive a kebab-case app id from a directory name.
# Lowercase, replace non-[a-z0-9] with '-', trim leading/trailing '-'.
nest_scaffold_derive_app_id() {
  local name="$1"
  echo "$name" | tr '[:upper:]' '[:lower:]' | sed -E 's/[^a-z0-9]+/-/g; s/^-+//; s/-+$//'
}

# Derive a title-cased display title from a kebab-case app id.
nest_scaffold_derive_app_title() {
  local app_id="$1"
  echo "$app_id" | sed -E 's/(^|-)([a-z])/\1\U\2/g; s/-/ /g'
}

# Resolve APP_NAME, APP_ID, and APP_TITLE from TARGET_DIR and TARGET_TITLE.
# Exits if APP_ID cannot be derived.
nest_scaffold_resolve_names() {
  APP_NAME="$(basename "$TARGET_DIR")"
  APP_ID="$(nest_scaffold_derive_app_id "$APP_NAME")"
  if [[ -z "$APP_ID" ]]; then
    echo "error: could not derive an app id from directory name '$APP_NAME'" >&2
    exit 1
  fi

  if [[ -n "${TARGET_TITLE:-}" ]]; then
    APP_TITLE="$TARGET_TITLE"
  else
    APP_TITLE="$(nest_scaffold_derive_app_title "$APP_ID")"
  fi
}

# Convert a kebab-case id to a snake_case Rust identifier.
nest_scaffold_derive_app_id_snake() {
  local app_id="$1"
  echo "$app_id" | sed -E 's/-/_/g'
}

# Replace {{app_id}}, {{app_id_snake}}, and {{display_title}} placeholders in a file.
nest_scaffold_replace_mustache() {
  local file="$1"
  local app_id="$2"
  local app_title="$3"
  local app_id_snake
  app_id_snake="$(nest_scaffold_derive_app_id_snake "$app_id")"
  sed -i \
    -e "s/{{app_id}}/${app_id}/g" \
    -e "s/{{app_id_snake}}/${app_id_snake}/g" \
    -e "s/{{display_title}}/${app_title}/g" \
    "$file"
}
