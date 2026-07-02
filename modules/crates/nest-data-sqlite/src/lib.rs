//! SQLite provider for the Nest data layer.
//!
//! Sync SQLite via rusqlite. Register with [`SqliteDataModule`] alongside
//! [`nest_data::DataModule`].

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

pub mod config;
pub mod connection;
mod error;
pub mod migration;
pub mod module;
pub mod notes;

pub use config::SqliteConfig;
pub use connection::SqliteConnection;
pub use migration::SqliteMigrationRunner;
pub use module::{SqliteDataModule, SQLITE_DATA_MODULE_ID};
pub use notes::{notes_migration, Note, NoteId, NotesRepository};

pub use nest_data::{DataModule, DataService, DATA_MODULE_ID};
pub use nest_error::{NestError, NestResult};
