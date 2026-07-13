//! Test-only helpers for spinning up disposable PostgreSQL containers.
//!
//! Not yet called anywhere — the tests that consume these (`connection.rs`,
//! `migration.rs`, `module.rs`, `vector.rs`, `notes.rs`) are retrofitted in
//! later, separate issues. Silencing `dead_code` here rather than in each
//! caller, since the "caller" doesn't exist yet.
#![cfg(test)]
#![allow(dead_code)]

use testcontainers_modules::postgres::Postgres as PostgresImage;
use testcontainers_modules::testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::{ContainerAsync, GenericImage, ImageExt};

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
    TestDb {
        _container: container,
        url,
    }
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
        .with_exposed_port(5432.tcp())
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_env_var("POSTGRES_PASSWORD", "postgres")
        .start()
        .await
        .expect("failed to start pgvector testcontainer");
    let host = container.get_host().await.expect("container host");
    let port = container
        .get_host_port_ipv4(5432)
        .await
        .expect("container port");
    let url = format!("postgres://postgres:postgres@{host}:{port}/postgres");
    TestVectorDb {
        _container: container,
        url,
    }
}
