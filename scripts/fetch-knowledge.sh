#!/usr/bin/env bash
# Fetch all nest-knowledge sources (Rust/egui git checkouts + webOS TV docs).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
KNOWLEDGE="${NEST_KNOWLEDGE:-/data/nest-knowledge}"
PYTHON="${ROOT}/.venv/bin/python"
FETCH_GIT=1
FETCH_WEBOS=1
FORCE=0

for arg in "$@"; do
  case "$arg" in
    --git-only) FETCH_WEBOS=0 ;;
    --webos-only) FETCH_GIT=0 ;;
    --force) FORCE=1 ;;
  esac
done

if [[ ! -x "$PYTHON" ]]; then
  echo "ERROR: .venv not found. Run:" >&2
  echo "  python3 -m venv .venv && .venv/bin/pip install -r tools/requirements.txt" >&2
  exit 1
fi

if ! command -v git >/dev/null 2>&1; then
  echo "ERROR: git is required to fetch Rust/egui knowledge sources." >&2
  exit 1
fi

mkdir -p "${KNOWLEDGE}" 2>/dev/null || true

if [[ ! -w "${KNOWLEDGE}" ]]; then
  echo "ERROR: knowledge directory is not writable: ${KNOWLEDGE}" >&2
  echo "  sudo chown \"\$USER\" \"${KNOWLEDGE}\"   # if created as root" >&2
  echo "  NEST_KNOWLEDGE=\$HOME/nest-knowledge ./scripts/fetch-knowledge.sh" >&2
  exit 1
fi

FETCH_ARGS=(--knowledge-root "${KNOWLEDGE}")
if [[ "$FORCE" -eq 1 ]]; then
  FETCH_ARGS+=(--force)
fi

if [[ "$FETCH_GIT" -eq 1 ]]; then
  echo "Fetching Rust / egui git sources into ${KNOWLEDGE} ..."
  "$PYTHON" "${ROOT}/tools/fetch_knowledge.py" "${FETCH_ARGS[@]}"
fi

if [[ "$FETCH_WEBOS" -eq 1 ]]; then
  mkdir -p "${KNOWLEDGE}/webos-tv"
  WEBOS_ARGS=()
  if [[ "$FORCE" -eq 1 ]]; then
    WEBOS_ARGS+=(--force)
  fi
  echo "Fetching webOS TV docs into ${KNOWLEDGE}/webos-tv ..."
  "$PYTHON" "${ROOT}/tools/fetch_webos_knowledge.py" \
    --config "${ROOT}/tools/webos-knowledge-urls.toml" \
    --output "${KNOWLEDGE}/webos-tv" \
    "${WEBOS_ARGS[@]}"
fi

echo ""
echo "Fetch complete. Index with:"
echo "  ./scripts/index-knowledge.sh --skip-fetch"
