# Phase 1 Task Spec — Harden & Retrofit `nest-data-postgres`

**Repo:** `pacificnm/nest` (framework repo, NOT the Sparrow repo)
**Crate:** `modules/crates/nest-data-postgres`
**Branch:** `feature/nest-data-postgres-hardening` (confirm branch/PR workflow with the repo owner before opening — do not push to `main` directly)

## Ground truth (read before starting, do not assume)

These are the actual current contents relevant to this task, verified against
the real repo. Do not "improve" anything not listed in the tasks below — this
is a shared module another product (Swift) already depends on.

- `src/config.rs` — `PostgresConfig { url: String, max_connections: u32 }`, `DEFAULT_MAX_CONNECTIONS: u32 = 5`, constructors `new()`, `from_env()`, `with_max_connections()`, `datasource()`.
- `src/connection.rs` — `PostgresConnection::connect(config) -> DataResult<Self>` and `connect_named(id, config) -> DataResult<Self>`, both currently call `PgPoolOptions::new().max_connections(...).connect(&config.url).await` **once, with no retry**. Has one `#[ignore = "requires DATABASE_URL and PostgreSQL"]` test: `health_check_live`.
- `src/module.rs` — `PostgresDataModule` registers the connection + runs migrations. Has two `#[ignore]` tests (around line 111 and 129).
- `src/migration.rs` — `apply_migrations(pool, migrations)`. Has one `#[ignore]` test (~line 152).
- `src/vector.rs` — `VectorSearch`, pgvector helpers. Has one `#[ignore]` test (~line 181) that additionally requires the `vector` Postgres extension.
- `src/notes.rs` — Swift-specific (`Note`, `NoteId`, `NotesRepository`) but lives in this crate. Has one `#[ignore]` test (~line 169). **Do not change its logic** — retrofit its test wiring only.
- `Cargo.toml` dev-dependencies currently: `nest-data = { workspace = true, features = ["async"] }`, `uuid = { version = "1", features = ["v4"] }`. No `testcontainers` yet.
- Current test convention across all six ignored tests: `std::env::var("DATABASE_URL").expect("DATABASE_URL")`, no automatic setup.

---

## Task 1 — Add connection retry/backoff to `PostgresConfig` / `PostgresConnection`

**File:** `modules/crates/nest-data-postgres/src/config.rs`

Add two fields with sensible defaults, plus builder methods, following the
exact style of the existing `with_max_connections`:

```rust
/// Default number of connection attempts before giving up.
pub const DEFAULT_CONNECT_RETRIES: u32 = 5;
/// Default initial backoff delay between connection attempts.
pub const DEFAULT_CONNECT_BACKOFF_MS: u64 = 200;
/// Default maximum backoff delay (backoff doubles each attempt, capped here).
pub const DEFAULT_CONNECT_BACKOFF_MAX_MS: u64 = 5_000;
```

Add to `PostgresConfig` struct: `pub connect_retries: u32`, `pub connect_backoff_ms: u64`, `pub connect_backoff_max_ms: u64`. Set these three defaults inside `PostgresConfig::new()`. Add builder methods `with_connect_retries(mut self, retries: u32) -> Self` and `with_connect_backoff(mut self, initial_ms: u64, max_ms: u64) -> Self`, same pattern as `with_max_connections`.

**File:** `modules/crates/nest-data-postgres/src/connection.rs`

Modify `connect_named` to retry with exponential backoff instead of a single attempt:

```rust
pub async fn connect_named(
    id: impl Into<ConnectionId>,
    config: &PostgresConfig,
) -> DataResult<Self> {
    let connection_id = id.into();
    let mut attempt: u32 = 0;
    let mut backoff_ms = config.connect_backoff_ms;
    let pool = loop {
        attempt += 1;
        match PgPoolOptions::new()
            .max_connections(config.max_connections)
            .connect(&config.url)
            .await
        {
            Ok(pool) => break pool,
            Err(err) if attempt < config.connect_retries => {
                tracing::warn!(
                    attempt,
                    max_attempts = config.connect_retries,
                    error = %err,
                    "postgres connection attempt failed, retrying"
                );
                tokio::time::sleep(std::time::Duration::from_millis(backoff_ms)).await;
                backoff_ms = (backoff_ms * 2).min(config.connect_backoff_max_ms);
            }
            Err(err) => return Err(map_sqlx_error(err)),
        }
    };
    // ... rest unchanged (ConnectionConfig::new, Ok(Self { ... }))
}
```

Notes:
- `map_sqlx_error` is already imported via `crate::error::sqlx_result` in this file — check whether `sqlx_result` or a bare `map_sqlx_error` function is exported from `error.rs` and use whichever is actually public (do not introduce a new error-mapping function).
- Add `tracing` as a dependency if it is not already present in `Cargo.toml` (`nest-ai-ollama`'s `Cargo.toml` uses `tracing`; check if `nest-data-postgres` already has it before adding — if it's already a transitive/workspace dependency, just add the `use` line).
- `tokio`'s `time` feature must be enabled. Current `Cargo.toml` has `tokio = { version = "1", features = ["rt-multi-thread", "macros"] }` — add `"time"` to that features list.

**New unit test** (in `connection.rs`, no `#[ignore]`, no live Postgres needed — connecting to a closed local port fails fast):

```rust
#[tokio::test]
async fn connect_retries_before_failing() {
    // Port 1 is a reserved/unlikely-to-be-open port; connection should fail fast each attempt.
    let config = PostgresConfig::new("postgresql://user:pass@127.0.0.1:1/db")
        .with_connect_retries(3)
        .with_connect_backoff(10, 50);
    let start = std::time::Instant::now();
    let result = PostgresConnection::connect(&config).await;
    assert!(result.is_err());
    // 3 attempts with backoff 10ms then 20ms between them = at least 30ms elapsed.
    assert!(start.elapsed() >= std::time::Duration::from_millis(30));
}
```

**Acceptance for Task 1:** `cargo test -p nest-data-postgres connect_retries_before_failing` passes with no Docker/Postgres running.

---

## Task 2 — Add `testcontainers` test infrastructure

**File:** `modules/crates/nest-data-postgres/Cargo.toml`

Add to `[dev-dependencies]`:

```toml
testcontainers-modules = { version = "0.14", features = ["postgres", "blocking"] }
```

Do not add a separate `testcontainers` dependency — `testcontainers-modules` re-exports it at an aligned version (confirmed from the crate's own docs); importing `testcontainers_modules::testcontainers::...` is correct and avoids a version-mismatch risk.

**New file:** `modules/crates/nest-data-postgres/src/test_support.rs` (module only compiled under `#[cfg(test)]` — add `#[cfg(test)] mod test_support;` to `lib.rs`)

This is the shared harness every retrofitted test will call. Two helpers —
plain Postgres for tests that don't need pgvector, and a pgvector-enabled
image for `vector.rs`'s test (the built-in `testcontainers_modules::postgres`
module is **plain Postgres, no pgvector** — confirmed from its own docs — so
the vector test needs a custom image, not the built-in module):

```rust
//! Test-only helpers for spinning up disposable PostgreSQL containers.
#![cfg(test)]

use testcontainers_modules::postgres::Postgres as PostgresImage;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::{ContainerAsync, GenericImage, ImageExt};
use testcontainers_modules::testcontainers::core::{IntoContainerPort, WaitFor};

/// Holds a running container alive for the test's duration; dropping it stops the container.
pub struct TestDb {
    _container: ContainerAsync<PostgresImage>,
    pub url: String,
}

/// Starts a plain PostgreSQL container (no pgvector) and returns a ready connection URL.
pub async fn start_postgres() -> TestDb {
    let container = PostgresImage::default()
        .start()
        .await
        .expect("failed to start postgres testcontainer");
    let host = container.get_host().await.expect("container host");
    let port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("container port");
    let url = format!("postgres://postgres:postgres@{host}:{port}/postgres");
    TestDb { _container: container, url }
}

/// Holds a running pgvector-enabled container alive for the test's duration.
pub struct TestVectorDb {
    _container: ContainerAsync<GenericImage>,
    pub url: String,
}

/// Starts a pgvector-enabled PostgreSQL container (`pgvector/pgvector:pg16`) and returns
/// a ready connection URL with the `vector` extension already installable.
pub async fn start_postgres_with_pgvector() -> TestVectorDb {
    let container = GenericImage::new("pgvector/pgvector", "pg16")
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_env_var("POSTGRES_PASSWORD", "postgres")
        .with_exposed_port(5432.tcp())
        .start()
        .await
        .expect("failed to start pgvector testcontainer");
    let host = container.get_host().await.expect("container host");
    let port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("container port");
    let url = format!("postgres://postgres:postgres@{host}:{port}/postgres");
    TestVectorDb { _container: container, url }
}
```

**Known risk to check before moving on:** the exact `WaitFor`/env-var API
shape and the `PostgresImage::default()` credentials may differ slightly by
`testcontainers-modules` version — run `cargo doc -p testcontainers-modules
--open` (or check docs.rs for the pinned version) and fix signatures if the
compiler disagrees with what's written above. Do not guess silently; if
something doesn't compile as written, that's expected — fix it against the
real API and note what changed in the PR description.

**Acceptance for Task 2:** `cargo build -p nest-data-postgres --tests` compiles (Docker does not need to be running yet for this step — only for actually running the retrofitted tests in Tasks 3–5).

---

## Task 3 — Retrofit `connection.rs`'s `health_check_live`

Replace:

```rust
#[tokio::test]
#[ignore = "requires DATABASE_URL and PostgreSQL"]
async fn health_check_live() {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let conn = PostgresConnection::connect(&PostgresConfig::new(url))
        .await
        .unwrap();
    let health = conn.health_check().unwrap();
    assert!(health.ok);
}
```

with:

```rust
#[tokio::test]
async fn health_check_live() {
    let db = crate::test_support::start_postgres().await;
    let conn = PostgresConnection::connect(&PostgresConfig::new(db.url))
        .await
        .unwrap();
    let health = conn.health_check().unwrap();
    assert!(health.ok);
}
```

No `#[ignore]`, no manual env var. **Acceptance:** `cargo test -p nest-data-postgres health_check_live` passes with Docker running, with zero manual setup steps beyond Docker being available.

---

## Task 4 — Retrofit `migration.rs`'s ignored test

Same pattern as Task 3: replace the `DATABASE_URL` env-var lookup with `let db = crate::test_support::start_postgres().await;` and use `db.url`. Remove the `#[ignore]` attribute. Do not change the test's actual assertions — only how it obtains a database.

**Acceptance:** `cargo test -p nest-data-postgres` (migration module) passes with Docker running.

---

## Task 5 — Retrofit `module.rs`'s two ignored tests

Same pattern. Both tests currently call `std::env::var("DATABASE_URL").expect("DATABASE_URL")` — replace both with `crate::test_support::start_postgres().await` and `db.url`. Remove both `#[ignore]` attributes.

**Acceptance:** both tests pass under `cargo test -p nest-data-postgres` with Docker running.

---

## Task 6 — Retrofit `vector.rs`'s ignored test

This one needs the **pgvector-enabled** container, not plain Postgres:

```rust
#[tokio::test]
async fn similarity_search_live() {
    let db = crate::test_support::start_postgres_with_pgvector().await;
    let conn = PostgresConnection::connect(&PostgresConfig::new(db.url))
        .await
        .unwrap();
    sqlx::query("CREATE EXTENSION IF NOT EXISTS vector")
        .execute(conn.pool())
        .await
        .unwrap();
    // ... keep the rest of the existing test body (table setup, insert, search_similar call,
    // assertions) exactly as it is today — only the connection setup changes.
}
```

Confirm the actual current test name and body in `vector.rs` before editing — do not assume it is literally named `similarity_search_live`; use whatever the existing `#[ignore]`d test is actually called and keep its body's logic intact.

**Acceptance:** the retrofitted vector test passes with Docker running, no manual `CREATE EXTENSION` step outside the test itself.

---

## Task 7 — Retrofit `notes.rs`'s ignored test

Same mechanical retrofit as Tasks 3–4 (plain `start_postgres()`, no
pgvector needed here based on `Note`/`NotesRepository` not being
vector-related). **Do not modify `Note`, `NoteId`, or `NotesRepository`
themselves** — this file is Swift's domain code living in a shared crate;
we're only touching how its test obtains a database connection.

**Acceptance:** test passes with Docker running.

---

## Task 8 — Verify no regressions

1. Run the full suite: `cargo test -p nest-data-postgres` (all tests, no `-- --ignored` needed anymore since nothing is `#[ignore]`d after Tasks 3–7).
2. Run Swift's own test suite (`cd apps/swift && ./build test`, or the equivalent per Swift's own docs) to confirm this module's public API changes (there are none — only `PostgresConfig` gained new fields with defaults, which is additive and non-breaking) don't break its consumer. **This step requires Docker as well**, since Swift's own tests likely depend on this same crate.
3. Confirm `cargo doc -p nest-data-postgres` still builds cleanly (the crate has `#![deny(missing_docs)]` — every new public item in Task 1 needs a doc comment, or the build will fail outright, not just warn).

**Acceptance for the whole Phase 1 spec:** all of the above pass, and
`git diff --stat` shows changes confined to `config.rs`, `connection.rs`,
`Cargo.toml`, the new `test_support.rs`, and the five retrofitted test
bodies — nothing else in the crate touched.

---

## Explicit "do not" list

- Do not touch `notes.rs`'s domain logic, `vector.rs`'s `VectorSearch` logic, or `migration.rs`'s `apply_migrations` logic — only test wiring and the two files in Task 1.
- Do not remove or rename any existing public item — this is a shared module, additive changes only.
- Do not skip Task 8's Swift regression check to save time — it's the actual point of doing the retrofit carefully instead of just deleting `#[ignore]` everywhere.
- Do not invent a `testcontainers` version if `0.14` turns out to be stale by the time this runs — check crates.io for whatever is current, but pin an exact version in `Cargo.toml` rather than leaving it unpinned.
