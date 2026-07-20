#!/usr/bin/env bash
# Scaffold a new multi-surface Nest product from templates/product/ into a target directory.
#
# Usage:
#   scripts/scaffold-product-app.sh <target-dir> [display-title]
#
# Example:
#   scripts/scaffold-product-app.sh apps/my-product "My Product"
#
# After scaffolding, cd into <target-dir> and run:
#   ./build desktop dev
#   ./build tui run
#   ./build cli run greet World
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# shellcheck source=/dev/null
source "$SCRIPT_DIR/nest-scaffold/lib.sh"

nest_scaffold_find_root "$SCRIPT_DIR"
TEMPLATE_DIR="$NEST_ROOT/templates/product"

nest_scaffold_parse_args "$@"

if [[ ! -d "$TEMPLATE_DIR" ]]; then
  echo "error: template not found at $TEMPLATE_DIR" >&2
  exit 1
fi

if [[ -e "$TARGET_DIR/Cargo.toml" || -e "$TARGET_DIR/crates" ]]; then
  echo "error: $TARGET_DIR already contains a Rust project — refusing to overwrite" >&2
  exit 1
fi

nest_scaffold_resolve_names

echo "Scaffolding Nest product:"
echo "  target:  $TARGET_DIR"
echo "  app id:  $APP_ID"
echo "  title:   $APP_TITLE"

mkdir -p "$TARGET_DIR"
# Copy contents including dotfiles (e.g. .gitignore).
cp -a "$TEMPLATE_DIR/." "$TARGET_DIR/"

# Replace mustache placeholders everywhere except binary assets.
while IFS= read -r -d '' file; do
  nest_scaffold_replace_mustache "$file" "$APP_ID" "$APP_TITLE"
done < <(find "$TARGET_DIR" -type f \
  ! -path '*/icons/*' \
  ! -path '*/node_modules/*' \
  ! -path '*/target/*' \
  ! -path '*/dist/*' \
  -print0)

chmod +x "$TARGET_DIR/build"
chmod +x "$TARGET_DIR/desktop/build"
chmod +x "$TARGET_DIR/tui/build"
chmod +x "$TARGET_DIR/cli/build"

echo
echo "Done. Next steps:"
echo "  cd $TARGET_DIR"
echo "  ./build desktop dev                          # desktop app"
echo "  ./build tui run                              # TUI app"
echo "  ./build cli run greet World                  # CLI app"
