# Services

nest-core v1 uses a **small typed service registry**. Services are singleton instances registered explicitly at build time and looked up by concrete type at runtime.

## The Service trait

`Service` is a marker trait. Any type that is `Send + Sync + 'static` automatically implements it:

```rust
pub trait Service: Send + Sync + 'static {}
impl<T: Send + Sync + 'static> Service for T {}
```

No methods are required. This keeps registration unconstrained — your service types can have any API you need.

## Registration rules (v1)

| Rule | Detail |
|------|--------|
| Lifetime | Singleton only — one instance per concrete type |
| Thread safety | `Send + Sync` required |
| Lifetime bound | `'static` required |
| Registration | Explicit — pass an instance to `register_service` |
| Lookup | By concrete type — `ctx.service::<GitService>()?` |
| Post-build | Registry is frozen — no registration after `build()` |

## ServiceRegistry

`ServiceRegistry` is the low-level storage type. It is used internally by `AppBuilder` and `AppContext`, but is also public if you need it directly.

```rust
use nest_core::{ServiceRegistry, NestError};

let mut registry = ServiceRegistry::new();

registry.register(Logger { prefix: "nest".into() })?;
registry.register(Settings { theme: "dark".into() })?;

let logger = registry.get::<Logger>()?;
assert!(registry.contains::<Settings>());
```

### Storage model

Services are stored in a `HashMap<TypeId, Box<dyn Any + Send + Sync>>`. Registration and lookup use Rust's type identity (`TypeId::of::<T>()`), not string names.

### Error behavior

| Operation | Error |
|-----------|-------|
| Register duplicate type | `NestError::ServiceAlreadyRegistered` |
| Get unregistered type | `NestError::ServiceNotFound` |

Error messages include the Rust type name (via `type_name::<T>()`) for debugging.

```rust
let err = registry.get::<MissingService>().unwrap_err();
// NestError::ServiceNotFound("my_app::MissingService")
```

## AppContext lookup

At runtime, use `AppContext` rather than `ServiceRegistry` directly:

```rust
pub fn handle_action(ctx: &AppContext) -> nest_core::NestResult<()> {
    let git = ctx.service::<GitService>()?;
    let settings = ctx.service::<SettingsService>()?;
    // ...
    Ok(())
}
```

`AppContext` also provides `has_service::<T>()` for optional checks without error handling:

```rust
if ctx.has_service::<GitService>() {
    let git = ctx.service::<GitService>()?;
}
```

## What v1 does not support

These are intentional omissions. They may be added in later versions:

| Feature | Status |
|---------|--------|
| Constructor injection | Not planned for v1 |
| Service factories / lazy init | Not planned for v1 |
| Scoped or transient lifetimes | Deferred |
| Trait-object lookup (`dyn Repository`) | Deferred to v2 |
| Named service instances (multiple per type) | Not supported |
| `remove` / replace at runtime | Not supported |

### Future: trait-object lookup

v2 may add registration by trait:

```rust
// Not available in v1
app.register_service_as::<dyn Repository, SqliteRepository>(repo);
let repo = ctx.service::<dyn Repository>()?;
```

## Testing services

Register services in tests without a full application:

```rust
#[test]
fn git_service_returns_repo() {
    let mut registry = ServiceRegistry::new();
    registry.register(GitService { repo: "/tmp".into() }).unwrap();

    let git = registry.get::<GitService>().unwrap();
    assert_eq!(git.repo, "/tmp");
}
```

For integration tests with modules and lifecycle, see the tests in [`builder.rs`](../../core/crates/nest-core/src/builder.rs).
