//! PostgreSQL provider for the Nest data layer.
//!
//! Async PostgreSQL via sqlx + Tokio with pgvector helpers. Register with
//! [`PostgresDataModule`] alongside [`nest_data::DataModule`].

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

pub mod config;
pub mod connection;
mod error;
pub mod migration;
pub mod module;
pub mod notes;
mod runtime;
#[cfg(test)]
mod test_support;
pub mod vector;

pub use config::PostgresConfig;
pub use connection::PostgresConnection;
pub use migration::PostgresMigrationRunner;
pub use module::{PostgresDataModule, POSTGRES_DATA_MODULE_ID};
pub use notes::{notes_migration, Note, NoteId, NotesRepository};
pub use vector::{
    enable_vector_migration, vector_samples_migration, SimilarityHit, VectorSearch,
    DEFAULT_EMBEDDING_DIM,
};

pub use nest_data::{DataModule, DataService, DATA_MODULE_ID};
pub use nest_error::{NestError, NestResult};
pub use sqlx::PgPool;
