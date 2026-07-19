#!/usr/bin/env bash
# Scaffold a new Nest CLI app (Rust binary) from templates/cli/ into a target directory.
# Usage:
#   scripts/scaffold-cli-app.sh <target-dir> [display-title]
# Example:
#   scripts/scaffold-cli-app.sh apps/my-cli "My CLI"

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TEMPLATE_DIR="$ROOT/templates/cli"

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "Usage: $0 <target-dir> [display-title]" >&2
  exit 1
fi

TARGET_DIR="$1"
if [[ "$TARGET_DIR" != /* ]]; then
  TARGET_DIR="$(pwd)/$TARGET_DIR"
fi

if [[ -e "$TARGET_DIR/src" || -e "$TARGET_DIR/Cargo.toml" ]]; then
  echo "error: $TARGET_DIR already contains a Rust project — refusing to overwrite" >&2
  exit 1
fi

APP_NAME="$(basename "$TARGET_DIR")"
# Derive kebab‑case ID (lowercase, non alphanum -> '-')
APP_ID="$(echo "$APP_NAME" | tr '[:upper:]' '[:lower:]' | sed -E 's/[^a-z0-9]+/-/g; s/^-+//; s/-+$//')"
if [[ -z "$APP_ID" ]]; then
  echo "error: could not derive an app id from directory name '$APP_NAME'" >&2
  exit 1
fi

# Determine title
if [[ -n "${2:-}" ]]; then
  APP_TITLE="$2"
else
  APP_TITLE="$(echo "$APP_ID" | sed -E 's/(^|-)([a-z])/\1\U\2/g; s/-/ /g')"
fi

mkdir -p "$TARGET_DIR"
# Copy source, Cargo.toml, README, and build script
cp -r "$TEMPLATE_DIR/src" "$TARGET_DIR/src"
cp "$TEMPLATE_DIR/Cargo.toml" "$TARGET_DIR/Cargo.toml"
cp "$TEMPLATE_DIR/README.md" "$TARGET_DIR/README.md"
if [[ -f "$TEMPLATE_DIR/build" ]]; then
  cp "$TEMPLATE_DIR/build" "$TARGET_DIR/build"
  chmod +x "$TARGET_DIR/build"
fi

# Replace placeholders in Cargo.toml ({{app_id}})
sed -i "s/{{app_id}}/$APP_ID/g" "$TARGET_DIR/Cargo.toml"

# Update README title (first line)
sed -i "1s/.*/# $APP_TITLE/" "$TARGET_DIR/README.md"

cat <<EOF
Scaffolded CLI app:
  target: $TARGET_DIR
  app id: $APP_ID
  title: $APP_TITLE
EOF
