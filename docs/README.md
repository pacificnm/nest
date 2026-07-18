# Nest Documentation Index

---

## Table of Contents


### Naming & Migration Guides

- **[Project Naming Conventions](./nest-naming.md)** - Official product names, application naming conventions, terminology guide for desktop shell applications (Tauri + React), UI components library (`@nest/components`), CLI and TUI apps
- **[MCP Migration Guide](./mcp-remote-migration.md)** - Remote Postgres + Ollama embeddings setup including schema changes, code updates, reindexing procedure with rollback notes


### Repository Layers & Architecture

#### Core Architecture Principles

- **[Repository Layering](./architecture.md#repository-layout-layers)** - Apps → Modules → Core dependency direction; layering rules and workspace membership section
- **[App Runtime Model](./app-standard.md)** - Runtime stack diagrams, host matrix (desktop GUI / CLI TUI / server), IPC boundaries at webview only for desktop hosts, shared bootstrap pattern, command surface API


#### Build System

- **[Build Commands](./build.md)** - Single `./build` interface across all app types with profiles (rust/server, tauri/desktop, node/client); commands: build / run / dev / test / clean / check; profile selection via env and flags