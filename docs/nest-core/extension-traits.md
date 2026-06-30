# Extension traits

nest-core defines minimal traits that optional Nest crates implement. This lets `AppBuilder` expose stable registration APIs without depending on UI, commands, or task crates.

In v1, panel, command, and job registration is **collect-only** — metadata is stored for introspection. Full registries and behavior live in downstream crates.

## Panel

Implemented by `nest-ui` and `nest-docking`.

```rust
pub trait Panel: Send + Sync + 'static {
    fn id(&self) -> &str;
}
```

```rust
struct ExplorerPanel {
    id: String,
}

impl Panel for ExplorerPanel {
    fn id(&self) -> &str {
        &self.id
    }
}

app.register_panel(ExplorerPanel { id: "explorer".into() });
```

## Command

Implemented by `nest-commands`.

```rust
pub trait Command: Send + Sync + 'static {
    fn id(&self) -> &str;
    fn title(&self) -> &str;
}
```

```rust
struct OpenFileCommand {
    id: String,
    title: String,
}

impl Command for OpenFileCommand {
    fn id(&self) -> &str { &self.id }
    fn title(&self) -> &str { &self.title }
}

app.register_command(OpenFileCommand {
    id: "file.open".into(),
    title: "Open File".into(),
});
```

## Job

Stub for [`nest-task`](../nest-task/README.md). nest-core defines metadata only; execution lives in `nest-task-runtime`.

```rust
pub trait Job: Send + Sync + 'static {
    fn id(&self) -> &str;
}
```

```rust
struct IndexWorkspaceJob {
    id: String,
}

impl Job for IndexWorkspaceJob {
    fn id(&self) -> &str { &self.id }
}

app.register_job(IndexWorkspaceJob { id: "index.workspace".into() });
```

## Plugin

See [Modules & plugins](modules.md). Plugins call `AppBuilder` registration methods from `register()`.

## RegistrationInfo

When panels, commands, or jobs are registered, nest-core stores `RegistrationInfo`:

```rust
pub struct RegistrationInfo {
    pub id: String,
    pub type_id: TypeId,
    pub type_name: String,
}
```

Access collected metadata before build:

```rust
app.register_panel(ExplorerPanel { id: "explorer".into() });

for info in app.panels() {
    println!("Panel: {} ({})", info.id, info.type_name);
}
```

`RegistrationInfo::new::<T>(id)` captures the concrete Rust type for debugging and future tooling (command palettes, panel managers, etc.).

## v1 vs future behavior

| Trait | v1 (`nest-core`) | Future crate |
|-------|------------------|--------------|
| `Panel` | Collect metadata | `nest-ui`, `nest-docking` render and manage layout |
| `Command` | Collect metadata | `nest-commands` binds shortcuts, palette, menus |
| `Job` | Collect metadata | [`nest-task-runtime`](../nest-task-runtime/README.md) runs tasks via `TaskManager` |
| `Plugin` | Delegates to `AppBuilder` | `nest-plugins` may add dynamic loading |

## Implementing an extension trait

When creating a new Nest crate:

1. Implement the appropriate trait on your types.
2. Register them in your module's `configure` or plugin's `register`.
3. Store runtime state as a **service** if other components need access.

```rust
struct GitPanel {
    panel_id: String,
}

impl Panel for GitPanel {
    fn id(&self) -> &str { &self.panel_id }
}

impl Module for GitModule {
    fn configure(&self, app: &mut AppBuilder) {
        app.register_service(GitService::new()).unwrap();
        app.register_panel(GitPanel { panel_id: "git".into() });
    }
}
```
