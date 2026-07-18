#!/usr/bin/env bash
# Nest standard build library — source from ./build in any app or the framework root.
#
# Set NEST_BUILD_PROFILE and profile-specific vars, then call nest_build_main "$@".

nest_build_usage() {
  cat <<'EOF'
Usage: ./build [command] [options]

Commands (same meaning everywhere):
  build   Production artifacts (default)
  run     Build if needed, then launch the app
  dev     Development mode (hot reload / cargo run)
  test    Run tests
  check   CI-style checks (fmt, clippy, tests)
  clean   Remove build artifacts

Examples:
  ./build              # same as ./build build
  ./build run
  ./build dev
  ./build test
  ./build run --release -- -- --config config.toml
EOF
}

nest_build_ensure_ui_deps() {
  local ui_dir="${NEST_UI_DIR:-ui}"
  if [[ ! -d "$APP_ROOT/$ui_dir/node_modules" ]]; then
    npm install --prefix "$APP_ROOT/$ui_dir"
  fi
  nest_build_ensure_local_npm_deps "$APP_ROOT/$ui_dir"
}

# Locally path-referenced `@nest/*` packages (e.g. `@nest/components` ->
# `../../../core/crates/nest-react-components`) ship raw TypeScript source,
# not a bundled dist — their own bare imports (`import { clsx } from
# "clsx"`) resolve relative to wherever that source *really* lives on disk
# (npm symlinks it there, it isn't copied), which only works if
# `npm install` has been run directly inside that package at least once.
# `npm install --prefix ui_dir` does NOT do this on the consumer's behalf —
# npm does not install a symlinked local dependency's own dependencies into
# the consumer's tree. Every app hitting this independently (confirmed with
# apps/sparrow's desktop UI, and separately by another tester on the plain
# `templates/desktop` template) is what this function exists to prevent.
nest_build_ensure_local_npm_deps() {
  local ui_dir="$1"
  local scoped_dir="$ui_dir/node_modules/@nest"
  [[ -d "$scoped_dir" ]] || return 0

  local link real_target
  for link in "$scoped_dir"/*; do
    [[ -e "$link" ]] || continue
    [[ -L "$link" ]] || continue
    real_target="$(cd "$(dirname "$link")" && readlink -f "$(basename "$link")")"
    if [[ -f "$real_target/package.json" && ! -d "$real_target/node_modules" ]]; then
      echo "nest-build: installing dependencies for locally-linked $(basename "$link") ($real_target)"
      npm install --prefix "$real_target"
    fi
  done
}

nest_build_rust_packages() {
  if [[ -n "${NEST_RUST_PACKAGES:-}" ]]; then
    echo "$NEST_RUST_PACKAGES"
  elif [[ -n "${NEST_RUST_PACKAGE:-}" ]]; then
    echo "$NEST_RUST_PACKAGE"
  else
    echo ""
  fi
}

nest_build_cargo_manifest_args() {
  if [[ -n "${NEST_CARGO_MANIFEST:-}" ]]; then
    echo --manifest-path "$APP_ROOT/$NEST_CARGO_MANIFEST"
  fi
}

nest_build_cargo() {
  local subcmd=$1
  shift
  local -a manifest_args=()
  read -r -a manifest_args <<< "$(nest_build_cargo_manifest_args)"
  local packages
  packages="$(nest_build_rust_packages)"
  if [[ -n "$packages" ]]; then
    local -a pkg_args=()
    for pkg in $packages; do
      pkg_args+=(-p "$pkg")
    done
    cargo "$subcmd" "${manifest_args[@]}" "${pkg_args[@]}" "$@"
  else
    cargo "$subcmd" "${manifest_args[@]}" "$@"
  fi
}

nest_build_release() {
  nest_build_cargo build --release "$@"
}

nest_build_debug() {
  nest_build_cargo build "$@"
}

nest_build_run_binary() {
  local profile=${1:-release}
  shift
  local bin_name="${NEST_RUST_BIN:-}"
  if [[ -z "$bin_name" ]]; then
    echo "error: NEST_RUST_BIN is not set for run" >&2
    exit 1
  fi
  local bin="$APP_ROOT/target/$profile/$bin_name"
  if [[ ! -x "$bin" ]]; then
    if [[ "$profile" == release ]]; then
      nest_build_release
    else
      nest_build_debug
    fi
  fi
  local -a config_args=()
  if [[ -n "${NEST_CONFIG_FILE:-}" && -f "$APP_ROOT/$NEST_CONFIG_FILE" ]]; then
    config_args=(--config "$APP_ROOT/$NEST_CONFIG_FILE")
  elif [[ -f "$APP_ROOT/config.toml" ]]; then
    config_args=(--config "$APP_ROOT/config.toml")
  fi
  if [[ "${1:-}" == "--" ]]; then
    shift
  fi
  exec "$bin" "${config_args[@]}" "$@"
}

nest_build_tauri_dev() {
  nest_build_ensure_ui_deps
  if [[ -n "${NEST_TAURI_NPM_SCRIPT:-}" ]]; then
    npm run "$NEST_TAURI_NPM_SCRIPT" --prefix "$APP_ROOT/${NEST_UI_DIR:-ui}" "$@"
  else
    npm run tauri:dev --prefix "$APP_ROOT/${NEST_UI_DIR:-ui}" "$@"
  fi
}

nest_build_tauri_bundle() {
  nest_build_ensure_ui_deps
  if [[ -n "${NEST_TAURI_NPM_SCRIPT_BUILD:-}" ]]; then
    npm run "$NEST_TAURI_NPM_SCRIPT_BUILD" --prefix "$APP_ROOT/${NEST_UI_DIR:-ui}" "$@"
  else
    npm run tauri:build --prefix "$APP_ROOT/${NEST_UI_DIR:-ui}" "$@"
  fi
}

nest_build_tauri_binary() {
  nest_build_ensure_ui_deps
  npm run build --prefix "$APP_ROOT/${NEST_UI_DIR:-ui}"
  nest_build_release "$@"
}

nest_build_profile_main() {
  local cmd=$1
  shift

  case "$NEST_BUILD_PROFILE" in
    workspace)
      case "$cmd" in
        build) nest_build_cargo build --workspace "$@" ;;
        run)
          echo "error: ./build run is not defined for the Nest framework workspace" >&2
          echo "       Use ./build build or cargo run -p <crate>" >&2
          exit 1
          ;;
        dev)
          echo "error: ./build dev is not defined for the Nest framework workspace" >&2
          exit 1
          ;;
        test) nest_build_cargo test --workspace "$@" ;;
        check)
          nest_build_cargo fmt --all -- --check
          nest_build_cargo clippy --workspace -- -D warnings
          nest_build_cargo test --workspace "$@"
          ;;
        clean) rm -rf "$APP_ROOT/target" ;;
        *) nest_build_usage; exit 1 ;;
      esac
      ;;
    rust)
      case "$cmd" in
        build) nest_build_release "$@" ;;
        release) nest_build_release "$@" ;;
        run)
          local profile=debug
          if [[ "${1:-}" == "--release" ]]; then
            profile=release
            shift
          fi
          nest_build_run_binary "$profile" "$@"
          ;;
        dev)
          if [[ "${1:-}" == "--release" ]]; then
            nest_build_cargo run --release "${@:2}"
          else
            nest_build_cargo run "$@"
          fi
          ;;
        test) nest_build_cargo test "$@" ;;
        check)
          nest_build_cargo fmt --all -- --check
          nest_build_cargo clippy -- -D warnings
          nest_build_cargo test -- --test-threads=1 "$@"
          nest_build_release
          ;;
        clean) rm -rf "$APP_ROOT/target" ;;
        *) nest_build_usage; exit 1 ;;
      esac
      ;;
    tauri)
      case "$cmd" in
        build)
          if [[ "${NEST_TAURI_MODE:-binary}" == bundle ]]; then
            nest_build_tauri_bundle "$@"
          else
            nest_build_tauri_binary "$@"
          fi
          ;;
        release) nest_build_tauri_bundle "$@" ;;
        run)
          if [[ "${NEST_TAURI_MODE:-binary}" == bundle ]]; then
            nest_build_tauri_bundle "$@"
            echo "Bundle written under target/release/bundle/"
          else
            nest_build_tauri_binary "$@"
            nest_build_run_binary release "$@"
          fi
          ;;
        dev) nest_build_tauri_dev "$@" ;;
        test)
          nest_build_cargo test "$@"
          if [[ -f "$APP_ROOT/${NEST_UI_DIR:-ui}/package.json" ]] \
            && grep -q '"test"' "$APP_ROOT/${NEST_UI_DIR:-ui}/package.json"; then
            npm run test --prefix "$APP_ROOT/${NEST_UI_DIR:-ui}" "$@"
          fi
          ;;
        check)
          nest_build_cargo fmt --all -- --check
          nest_build_cargo clippy -- -D warnings
          nest_build_cargo test -- --test-threads=1 "$@"
          npm run build --prefix "$APP_ROOT/${NEST_UI_DIR:-ui}"
          ;;
        clean)
          rm -rf "$APP_ROOT/target" \
            "$APP_ROOT/${NEST_UI_DIR:-ui}/dist" \
            "$APP_ROOT/${NEST_UI_DIR:-ui}/node_modules"
          ;;
        *) nest_build_usage; exit 1 ;;
      esac
      ;;
    node)
      case "$cmd" in
        build)
          nest_build_ensure_ui_deps
          npm run build --prefix "$APP_ROOT/${NEST_UI_DIR:-.}" "$@"
          ;;
        run)
          nest_build_ensure_ui_deps
          npm run preview --prefix "$APP_ROOT/${NEST_UI_DIR:-.}" "$@"
          ;;
        dev)
          nest_build_ensure_ui_deps
          npm run dev --prefix "$APP_ROOT/${NEST_UI_DIR:-.}" "$@"
          ;;
        test)
          nest_build_ensure_ui_deps
          npm run test --prefix "$APP_ROOT/${NEST_UI_DIR:-.}" "$@"
          ;;
        check)
          nest_build_ensure_ui_deps
          npm run build --prefix "$APP_ROOT/${NEST_UI_DIR:-.}" "$@"
          ;;
        clean)
          rm -rf "$APP_ROOT/${NEST_UI_DIR:-.}/dist" \
            "$APP_ROOT/${NEST_UI_DIR:-.}/node_modules"
          ;;
        *) nest_build_usage; exit 1 ;;
      esac
      ;;
    *)
      echo "error: unknown NEST_BUILD_PROFILE=${NEST_BUILD_PROFILE:-}" >&2
      exit 1
      ;;
  esac
}

nest_build_main() {
  APP_ROOT="${APP_ROOT:-$(cd "$(dirname "${BASH_SOURCE[1]}")" && pwd)}"
  export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$APP_ROOT/target}"
  export RUSTFLAGS="${RUSTFLAGS:--D warnings}"

  local cmd="${1:-build}"
  if [[ "$cmd" == -h || "$cmd" == --help || "$cmd" == help ]]; then
    nest_build_usage
    exit 0
  fi
  shift || true
  nest_build_profile_main "$cmd" "$@"
}
