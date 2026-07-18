#!/usr/bin/env bash
# Scaffold a new Nest desktop app (Tauri + React + Tailwind) from
# templates/desktop/ into a target directory — typically apps/<name>/.
#
# Usage:
#   scripts/scaffold-desktop-app.sh <target-dir> [display-title]
#
# Example:
#   scripts/scaffold-desktop-app.sh apps/pigion "Pigion"
#
# After scaffolding, cd into <target-dir> and run ./build dev.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
NEST_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TEMPLATE_DIR="$NEST_ROOT/templates/desktop"

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "Usage: $0 <target-dir> [display-title]" >&2
  exit 1
fi

TARGET_DIR="$1"
if [[ "$TARGET_DIR" != /* ]]; then
  TARGET_DIR="$(pwd)/$TARGET_DIR"
fi

if [[ ! -d "$TEMPLATE_DIR" ]]; then
  echo "error: template not found at $TEMPLATE_DIR" >&2
  exit 1
fi

if [[ -e "$TARGET_DIR/ui" || -e "$TARGET_DIR/src-tauri" ]]; then
  echo "error: $TARGET_DIR already has ui/ or src-tauri/ — refusing to overwrite" >&2
  echo "       remove them first if you want to re-scaffold" >&2
  exit 1
fi

app_name="$(basename "$TARGET_DIR")"

# Sanitize to a kebab-case identifier: lowercase, non [a-z0-9-] -> '-',
# squeeze repeats, trim leading/trailing '-'.
app_id="$(echo "$app_name" | tr '[:upper:]' '[:lower:]' | sed -E 's/[^a-z0-9]+/-/g; s/^-+//; s/-+$//')"
if [[ -z "$app_id" ]]; then
  echo "error: could not derive an app id from directory name '$app_name'" >&2
  exit 1
fi

# Title: explicit arg, or Title Case each hyphen-separated word of app_id.
if [[ -n "${2:-}" ]]; then
  app_title="$2"
else
  app_title="$(echo "$app_id" | sed -E 's/(^|-)([a-z])/\1\U\2/g; s/-/ /g')"
fi

tauri_identifier="com.nest.$app_id"
cache_dir_name="nest-$app_id-cache"
ui_package_name="$app_id-ui"

echo "Scaffolding desktop app:"
echo "  target:  $TARGET_DIR"
echo "  app id:  $app_id"
echo "  title:   $app_title"
echo "  bundle:  $tauri_identifier"

mkdir -p "$TARGET_DIR"
cp -r "$TEMPLATE_DIR/ui" "$TARGET_DIR/ui"
cp -r "$TEMPLATE_DIR/src-tauri" "$TARGET_DIR/src-tauri"
cp "$TEMPLATE_DIR/build" "$TARGET_DIR/build"
cp "$TEMPLATE_DIR/nest-app.toml" "$TARGET_DIR/nest-app.toml"
cp "$TEMPLATE_DIR/.gitignore" "$TARGET_DIR/.gitignore"
chmod +x "$TARGET_DIR/build"

# Remove template build artifacts that shouldn't have been there but are
# harmless to double-check for (tsbuildinfo, in case the template ever
# picks one up again).
rm -f "$TARGET_DIR/ui/tsconfig.tsbuildinfo"

replace_in_file() {
  local file="$1"
  sed -i \
    -e "s/nest-desktop-template-ui/${ui_package_name}/g" \
    -e "s/nest-desktop-template-cache/${cache_dir_name}/g" \
    -e "s/com\\.nest\\.desktop-template/${tauri_identifier}/g" \
    -e "s/nest-desktop-template/${app_id}/g" \
    -e "s/Nest Desktop Template/${app_title}/g" \
    "$file"
}

replace_in_file "$TARGET_DIR/src-tauri/Cargo.toml"
sed -i "s/Nest desktop app template (Tauri + React)/${app_title} desktop app (Tauri + React)/" "$TARGET_DIR/src-tauri/Cargo.toml"
replace_in_file "$TARGET_DIR/src-tauri/tauri.conf.json"
replace_in_file "$TARGET_DIR/src-tauri/src/main.rs"
replace_in_file "$TARGET_DIR/ui/package.json"
replace_in_file "$TARGET_DIR/ui/package-lock.json"
replace_in_file "$TARGET_DIR/ui/index.html"
replace_in_file "$TARGET_DIR/ui/src/App.tsx"

# nest-app.toml uses a generic placeholder, not the template name.
sed -i \
  -e "s/name = \"My App\"/name = \"${app_title}\"/" \
  -e "s#Short description shown when the app opens in Nest Shell\\.#${app_title} desktop app.#" \
  "$TARGET_DIR/nest-app.toml"

cat > "$TARGET_DIR/README.md" <<EOF
# ${app_title}

Nest desktop app (Tauri + React + Tailwind), scaffolded from
[\`templates/desktop\`](../../templates/desktop) by
\`scripts/scaffold-desktop-app.sh\`.

\`\`\`bash
./build dev      # hot reload for Tauri + Vite
./build run      # production build + launch
./build build    # production artifacts only
\`\`\`

See [Nest build standard](../../docs/build.md) and
[nest-tauri docs](../../docs/nest-tauri/README.md).
EOF

echo
echo "Done. Next steps:"
echo "  cd $TARGET_DIR"
echo "  ./build dev"
