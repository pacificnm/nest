#!/usr/bin/env bash
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"
PYTHONPATH=tools exec .venv/bin/python tools/memory_hooks.py stop
