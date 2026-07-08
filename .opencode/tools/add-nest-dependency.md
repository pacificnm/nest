# Tool: add-nest-dependency

**Purpose**: Add a Nest shared crate dependency to an app or module correctly.

## When to invoke

- User wants to add a Nest crate to their app
- User is setting up a new app in `apps/`
- User needs to configure workspace dependencies

## Workflow

### Step 1: Identify the target

**Where is the user adding the dependency?**

| Location | Type | Dependency method |
|----------|------|-------------------|
| `apps/*/Cargo.toml` | App (Rust binary) | Git or path |
| `apps/*/src-tauri/Cargo.toml` | App (Tauri) | Git or path |
| `modules/crates/nest-*/Cargo.toml` | Module | Workspace |
| `core/crates/nest-*/Cargo.toml` | Core | Workspace |

### Step 2: Choose dependency method

#### For Apps (separate repos) - Git dependency (typical)

```toml
[dependencies]
nest-error = { git = "https://github.com/pacificnm/nest", branch = "main" }
nest-config = { git = "https://github.com/pacificnm/nest", branch = "main" }
```

#### For Apps (local checkout) - Path patch

```toml
# apps/my-app/.cargo/config.toml
[patch."https://github.com/pacificnm/nest"]
nest-core = { path = "../../../core/crates/nest-core" }
nest-error = { path = "../../../core/crates/nest-error" }
nest-config = { path = "../../../core/crates/nest-config" }
```

Then in `Cargo.toml`:

```toml
[dependencies]
nest-core = { git = "https://github.com/pacificnm/nest", branch = "main" }
nest-error = { git = "https://github.com/pacificnm/nest", branch = "main" }
```

The patch overrides to use local path.

#### For Core/Modules in nest monorepo - Workspace

```toml
[dependencies]
nest-error = { workspace = true }
nest-core = { workspace = true }
```

### Step 3: Add the dependency

**For workspace dependencies** (core/modules):

1. Ensure the crate is in root `Cargo.toml` `[workspace.dependencies]`
2. Add to crate's `Cargo.toml`: `nest-* = { workspace = true }`

**For app git dependencies**:

1. Add to app's `Cargo.toml` with git URL
2. Optionally set up `.cargo/config.toml` for local patches

### Step 4: Verify

```bash
# In the app/module/core directory
cargo check
cargo tree | grep nest-
```

## Common Scenarios

### Scenario 1: Adding error handling to an app

**User**: "I need to add NestError to my app"

**Actions**:
1. Add to `apps/my-app/src-tauri/Cargo.toml`:
   ```toml
   [dependencies]
   nest-error = { git = "https://github.com/pacificnm/nest", branch = "main" }
   ```
2. Or if using local checkout, add to `.cargo/config.toml`:
   ```toml
   [patch."https://github.com/pacificnm/nest"]
   nest-error = { path = "../../../core/crates/nest-error" }
   ```
3. Verify: `cargo check`

### Scenario 2: Adding multiple Nest crates to a new Tauri app

**User**: "Setting up a new desktop app with Tauri"

**Actions**:
1. Add to `apps/my-app/src-tauri/Cargo.toml`:
   ```toml
   [dependencies]
   nest-tauri = { git = "https://github.com/pacificnm/nest", branch = "main", features = ["runtime", "images"] }
   nest-core = { git = "https://github.com/pacificnm/nest", branch = "main" }
   nest-app = { git = "https://github.com/pacificnm/nest", branch = "main" }
   nest-config = { git = "https://github.com/pacificnm/nest", branch = "main" }
   nest-logging = { git = "https://github.com/pacificnm/nest", branch = "main" }
   nest-error = { git = "https://github.com/pacificnm/nest", branch = "main", features = ["serde"] }
   nest-design = { git = "https://github.com/pacificnm/nest", branch = "main" }
   nest-theme = { git = "https://github.com/pacificnm/nest", branch = "main" }
   nest-react-theme = { git = "https://github.com/pacificnm/nest", branch = "main" }
   ```

2. Set up local patch if needed

### Scenario 3: Module adding HTTP client

**User**: "My module needs to make HTTP requests"

**Actions**:
1. Add to `modules/crates/nest-mymodule/Cargo.toml`:
   ```toml
   [dependencies]
   nest-http-client = { workspace = true }
   nest-error = { workspace = true }
   ```
2. Verify: `cd modules/crates/nest-mymodule && cargo check`

## Crate-Specific Features

Some Nest crates have optional features:

```toml
nest-error = { workspace = true, features = ["serde", "diagnostics"] }
nest-config = { workspace = true, features = ["json"] }
nest-tauri = { workspace = true, features = ["runtime", "images", "async"] }
nest-data = { workspace = true, features = ["async"] }
nest-http = { workspace = true, features = ["serde"] }
```

## Verification Checklist

- [ ] Dependency added to correct `Cargo.toml`
- [ ] Using correct method (git vs workspace vs path)
- [ ] Features specified if needed
- [ ] `cargo check` passes
- [ ] No circular dependencies
- [ ] Follows architecture rules (core → modules → apps)

## Related Files

- `/data/projects/nest/Cargo.toml` - Workspace dependencies
- `/data/projects/nest/apps/README.md` - App setup guide
- `/data/projects/nest/docs/architecture.md` - Dependency rules
