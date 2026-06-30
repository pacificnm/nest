#!/usr/bin/env bash
# Create nest_memory schema as the postgres superuser.
#
# Uses stdin (not psql -f) so postgres can read the SQL even when your home
# directory is not world-accessible (common "Permission denied" with -f).
#
# Usage:
#   ./scripts/setup-database-postgres.sh
#   NEST_MEMORY_USER=jaimie ./scripts/setup-database-postgres.sh

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DB_NAME="${NEST_MEMORY_DB:-nest_memory}"
APP_USER="${NEST_MEMORY_USER:-$USER}"
SQL_FILE="$ROOT/tools/setup_database.sql"

if [[ ! -f "$SQL_FILE" ]]; then
  echo "ERROR: missing $SQL_FILE" >&2
  exit 1
fi

echo "Database: $DB_NAME"
echo "Grant tables to user: $APP_USER"

if ! command -v sudo >/dev/null 2>&1; then
  echo "ERROR: sudo is required to run commands as postgres." >&2
  exit 1
fi

if ! sudo -u postgres psql -tc "SELECT 1 FROM pg_database WHERE datname = '${DB_NAME}'" | grep -q 1; then
  echo "Creating database ${DB_NAME}..."
  sudo -u postgres createdb "$DB_NAME"
else
  echo "Database ${DB_NAME} already exists."
fi

sudo -u postgres createuser "$APP_USER" 2>/dev/null || true

echo "Applying schema (piped via stdin)..."
sed "s/REPLACE_WITH_OS_USER/${APP_USER}/g" "$SQL_FILE" | sudo -u postgres psql -v ON_ERROR_STOP=1 "$DB_NAME"

sudo -u postgres psql -d "$DB_NAME" -v ON_ERROR_STOP=1 -c \
  "GRANT ALL PRIVILEGES ON DATABASE ${DB_NAME} TO \"${APP_USER}\";"
sudo -u postgres psql -d "$DB_NAME" -v ON_ERROR_STOP=1 -c \
  "GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO \"${APP_USER}\";"
sudo -u postgres psql -d "$DB_NAME" -v ON_ERROR_STOP=1 -c \
  "GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO \"${APP_USER}\";"

echo ""
echo "Schema ready. Use in .env (peer auth as ${APP_USER}):"
echo "  DATABASE_URL=\"postgresql:///${DB_NAME}?host=/var/run/postgresql\""
echo ""
echo "Or TCP as postgres (if you use password auth):"
echo "  DATABASE_URL=\"postgresql://postgres:postgres@localhost/${DB_NAME}\""
