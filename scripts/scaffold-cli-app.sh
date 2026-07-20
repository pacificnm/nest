#!/usr/bin/env bash
# Scaffold a new Nest CLI app from templates/cli/ into a target directory.
#
# Usage:
#   scripts/scaffold-cli-app.sh <target-dir> [display-title]
# Example:
#   scripts/scaffold-cli-app.sh apps/my-cli "My CLI"
#
# After scaffolding, cd into <target-dir> and run ./build dev.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# shellcheck source=/dev/null
source "$SCRIPT_DIR/nest-scaffold/lib.sh"

nest_scaffold_find_root "$SCRIPT_DIR"
TEMPLATE_DIR="$NEST_ROOT/templates/cli"

nest_scaffold_parse_args "$@"

if [[ ! -d "$TEMPLATE_DIR" ]]; then
  echo "error: template not found at $TEMPLATE_DIR" >&2
  exit 1
fi

if [[ -e "$TARGET_DIR/crates" || -e "$TARGET_DIR/Cargo.toml" ]]; then
  echo "error: $TARGET_DIR already contains a Rust project — refusing to overwrite" >&2
  exit 1
fi

nest_scaffold_resolve_names

echo "Scaffolding CLI app:"
echo "  target:  $TARGET_DIR"
echo "  app id:  $APP_ID"
echo "  title:   $APP_TITLE"

mkdir -p "$TARGET_DIR"
# Copy contents including dotfiles (e.g. .gitignore).
cp -a "$TEMPLATE_DIR/." "$TARGET_DIR/"

# Replace placeholders in Cargo.toml files, source, build script, and README.
for file in \
  "$TARGET_DIR/Cargo.toml" \
  "$TARGET_DIR/crates/core/Cargo.toml" \
  "$TARGET_DIR/crates/core/src/lib.rs" \
  "$TARGET_DIR/crates/cli/Cargo.toml" \
  "$TARGET_DIR/crates/cli/src/main.rs" \
  "$TARGET_DIR/build" \
  "$TARGET_DIR/README.md"; do
  nest_scaffold_replace_mustache "$file" "$APP_ID" "$APP_TITLE"
done

chmod +x "$TARGET_DIR/build"

echo
echo "Done. Next steps:"
echo "  cd $TARGET_DIR"
echo "  ./build dev        # run in debug mode"
echo "  ./build run        # run in release mode"
echo "  ./build build      # build the release binary"
