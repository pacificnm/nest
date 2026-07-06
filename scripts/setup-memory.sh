#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

if [[ ! -x "$ROOT/.venv/bin/python" ]]; then
  echo "Creating .venv..."
  python3 -m venv "$ROOT/.venv" || {
    echo "ERROR: could not create .venv (install python3-venv if needed)." >&2
    exit 1
  }
fi

if ! "$ROOT/.venv/bin/python" -m pip --version >/dev/null 2>&1; then
  echo "Bootstrapping pip in .venv (ensurepip unavailable on this system)..."
  curl -sS https://bootstrap.pypa.io/get-pip.py -o /tmp/nest-get-pip.py
  "$ROOT/.venv/bin/python" /tmp/nest-get-pip.py
fi

if ! "$ROOT/.venv/bin/python" -c "import mcp, psycopg" >/dev/null 2>&1; then
  echo "Installing Python dependencies..."
  "$ROOT/.venv/bin/pip" install -r "$ROOT/tools/requirements.txt"
fi

if "$ROOT/.venv/bin/python" "$ROOT/tools/setup_database.py"; then
  :
else
  status=$?
  if [[ $status -ne 1 ]]; then
    exit $status
  fi
  echo "" >&2
  echo "Trying postgres superuser setup (pipes SQL via stdin)..." >&2
  "$ROOT/scripts/setup-database-postgres.sh"
fi

echo ""
echo "Verifying schema..."
"$ROOT/.venv/bin/python" "$ROOT/tools/verify_memory_schema.py"

echo ""
echo "Context memory smoke test..."
PYTHONPATH=tools "$ROOT/.venv/bin/python" - <<'PY'
from context_memory import save_context, search_context

entry_id = save_context(
    "setup smoke test",
    title="context memory check",
    session_key="setup",
)
hits = search_context("setup smoke test", session_key="setup")
print(f"saved entry id={entry_id}, search hits={len(hits)}")
PY

cat <<EOF

Next steps:
  1. cp .env.example .env   # set OPENAI_API_KEY (and DATABASE_URL if needed)
  2. ./scripts/index-memory.sh
  3. ./scripts/index-knowledge.sh   # fetches Rust/Tauri/React/Tailwind/webOS sources, then indexes
  4. Update paths in .cursor/mcp.json for your machine
  5. Reload Cursor (Developer: Reload Window)
  6. Verify Tools & MCP: nest-memory, nest-knowledge, nest-context-memory

See tools/MCP-SETUP.md for full setup and troubleshooting.
EOF
