#!/usr/bin/env bash
# Create agent_context_memory if missing (safe to re-run).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DB_NAME="${NEST_MEMORY_DB:-nest_memory}"
APP_USER="${NEST_MEMORY_USER:-$USER}"
SQL_FILE="$ROOT/tools/setup_context_memory.sql"

echo "Ensuring agent_context_memory in ${DB_NAME} for user ${APP_USER}..."

if "$ROOT/.venv/bin/python" "$ROOT/tools/setup_context_memory.py" 2>/dev/null; then
  echo "agent_context_memory ready (via app user)."
  exit 0
fi

echo "App user cannot CREATE tables; using postgres superuser..." >&2
sed "s/REPLACE_WITH_OS_USER/${APP_USER}/g" "$SQL_FILE" | sudo -u postgres psql -v ON_ERROR_STOP=1 "$DB_NAME"
sudo -u postgres psql -d "$DB_NAME" -v ON_ERROR_STOP=1 -c \
  "GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO \"${APP_USER}\";"
sudo -u postgres psql -d "$DB_NAME" -v ON_ERROR_STOP=1 -c \
  "GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO \"${APP_USER}\";"

echo "agent_context_memory ready."
