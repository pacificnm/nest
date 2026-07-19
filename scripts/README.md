# Scripts Overview

This directory contains utility scripts used to set up and manage the Nest development environment.

| Script | Description | Last Modified |
|--------|-------------|----------------|
| `setup-memory.sh` | Bootstraps a Python virtual environment, installs dependencies, sets up the Nest memory database schema, and runs a smoke test. | 2026-07-13 |
| `setup-context-memory.sh` | Ensures the `agent_context_memory` table exists, creating it if necessary, and grants appropriate privileges. | 2026-07-13 |
| `setup-database-postgres.sh` | Creates the `nest_memory` PostgreSQL schema as the superuser, grants privileges, and prints connection hints. | 2026-07-13 |
| `fetch-knowledge.sh` | Retrieves all external knowledge sources (Rust repos, Tauri/React/Tailwind docs, webOS TV docs) into the knowledge directory. | 2026-07-13 |
| `fetch-webos-knowledge.sh` | Fetches only the webOS TV documentation and indexes it as a dedicated collection. | 2026-07-13 |
| `index-knowledge.sh` | Generates a TOML config for knowledge collections, sets up the knowledge database, and runs the embedding indexer. | 2026-07-13 |
| `index-memory.sh` | Runs the memory indexing Python script to embed project documentation for semantic search. | 2026-07-13 |
| `scaffold-desktop-app.sh` | Scaffolds a new Nest desktop application (Tauri + React + Tailwind) from the `templates/desktop` template into a target directory. | 2026-07-18 |
| `nest-build/lib.sh` | Helper library for the Nest build system (used by the `./build` scripts in apps). | 2026-07-26 |

These scripts are intended to be run from the repository root.
