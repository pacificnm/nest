# Tool: check-nest-dependencies

**Purpose**: Verify that Nest crate dependencies follow the architecture rules and help users configure them correctly.

## Architecture Rules

```
Apps (apps/)
  ↓ may depend on
Modules (modules/crates/)
  ↓ may depend on
Core (core/crates/)
```

### Dependency Rules

| Layer | May depend on | Must NOT depend on |
|-------|---------------|-------------------|
| **Core** (`core/crates/`) | Other core crates | Modules, Apps |
| **Modules** (`modules/crates/`) | Core crates | Apps, other modules (avoid) |
| **Apps** (`apps/`) | Core + Modules | (nothing restricted) |

## When to invoke

- User is adding dependencies to a crate's `Cargo.toml`
- User asks if a dependency is allowed
- User is setting up a new crate or app
- Verifying architecture compliance

## How to check

### 1. Identify the crate's layer

```bash
# Check path
core/crates/nest-*    → Core layer
modules/crates/nest-* → Module layer
apps/*/               → App layer
```

### 2. Read the Cargo.toml

```toml
[dependencies]
nest-error = { workspace = true }  # ✓ OK for any layer
nest-data = { workspace = true }   # ✓ OK for modules and apps
my-app-code = { path = "../.." }   # ✗ NOT OK for core/modules
```

### 3. Validate against rules

**Core crate** (`core/crates/nest-*/Cargo.toml`):
- ✓ Can depend on: other `core/crates/*`
- ✗ Cannot depend on: `modules/crates/*`, `apps/*`

**Module crate** (`modules/crates/nest-*/Cargo.toml`):
- ✓ Can depend on: `core/crates/*`
- ⚠ Should avoid: other `modules/crates/*`
- ✗ Cannot depend on: `apps/*`

**App** (`apps/*/Cargo.toml` or `apps/*/src-tauri/Cargo.toml`):
- ✓ Can depend on: any core or module crate

## Common Patterns

### App adding Nest dependencies

```toml
# apps/my-app/src-tauri/Cargo.toml
[dependencies]
nest-core = { workspace = true }      # If using local checkout
nest-error = { workspace = true }
nest-config = { workspace = true }
nest-tauri = { workspace = true, features = ["runtime"] }

# Or via git (typical for separate app repos)
nest-core = { git = "https://github.com/pacificnm/nest", branch = "main" }
nest-error = { git = "https://github.com/pacificnm/nest", branch = "main" }
```

### Module depending on core

```toml
# modules/crates/nest-airtable/Cargo.toml
[dependencies]
nest-core = { workspace = true }
nest-http-client = { workspace = true }
nest-error = { workspace = true }
```

### Core crate depending on sibling

```toml
# core/crates/nest-data/Cargo.toml
[dependencies]
nest-core = { workspace = true }
nest-error = { workspace = true }
```

## Workspace Configuration

### Root workspace (nest monorepo)

```toml
# /data/projects/nest/Cargo.toml
[workspace]
members = [
  "core/crates/*",
  "modules/crates/*",
]

[workspace.dependencies]
nest-core = { path = "core/crates/nest-core" }
nest-error = { path = "core/crates/nest-error" }
# ... etc
```

### App with local checkout

```toml
# apps/my-app/.cargo/config.toml
[patch."https://github.com/pacificnm/nest"]
nest-core = { path = "../../../core/crates/nest-core" }
nest-error = { path = "../../../core/crates/nest-error" }
```

## Verification Commands

```bash
# Check for dependency cycles
cargo tree --depth 1

# Verify workspace dependencies
cargo metadata --format-version 1 | jq '.packages[] | select(.name | startswith("nest-")) | {name, dependencies}'

# Check specific crate dependencies
cd core/crates/nest-core && cargo tree
```

## Red Flags

🚩 Core crate depending on module:
```toml
# core/crates/nest-core/Cargo.toml
nest-airtable = { workspace = true }  # ✗ WRONG - airtable is a module
```

🚩 Module depending on app:
```toml
# modules/crates/nest-*/Cargo.toml
my-app-common = { path = "../../apps/my-app/common" }  # ✗ WRONG
```

🚩 Core crate depending on app:
```toml
# core/crates/nest-*/Cargo.toml
my-app-types = { path = "../../apps/my-app/types" }  # ✗ WRONG
```

## Related Files

- `/data/projects/nest/docs/architecture.md` - Full architecture documentation
- `/data/projects/nest/Cargo.toml` - Root workspace definition
- `/data/projects/nest/core/crates/*/Cargo.toml` - Core crate dependencies
- `/data/projects/nest/modules/crates/*/Cargo.toml` - Module crate dependencies
