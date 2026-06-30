//! Database-agnostic data contracts for the Nest framework.
//!
//! nest-data defines what data access means: repositories, transactions,
//! migrations, and connection lifecycle. Provider crates (e.g. `nest-data-sqlite`)
//! implement how I/O happens.
//!
//! Sync traits are always available. Async traits are behind the `async` feature.
//! nest-core registers [`DataService`] but does not execute database I/O.

#![deny(missing_docs)]
#![allow(clippy::result_large_err)]

pub mod codes;
mod config;
mod connection;
mod error;
mod migration;
mod module;
mod prelude;
mod query;
mod repository;
mod service;
mod transaction;

pub use config::{ConnectionConfig, ConnectionId, ProviderKind};
pub use connection::{ConnectionHealth, ConnectionRegistry, DataConnection};
pub use error::{DataError, DataErrorKind, DataResult};
pub use migration::{Migration, MigrationRunner, SqlMigration};
pub use module::{DataModule, DATA_MODULE_ID};
pub use query::ListQuery;
pub use repository::Repository;
pub use service::DataService;
pub use transaction::{Transaction, Transactional};

#[cfg(feature = "async")]
pub use repository::AsyncRepository;
#[cfg(feature = "async")]
pub use transaction::{AsyncTransaction, AsyncTransactional};

pub use nest_core::{Module, ModuleId};
pub use nest_error::{NestError, NestResult};

impl From<DataError> for NestError {
    fn from(error: DataError) -> Self {
        let mut nest_error = NestError::data(error.message())
            .with_code(error.nest_code())
            .with_module("nest-data");

        if let Some(connection) = error.connection_id() {
            nest_error = nest_error.with_operation(format!("connection: {connection}"));
        }

        nest_error.with_source(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nest_error::NestErrorKind;

    #[test]
    fn data_error_converts_to_nest_error() {
        let err = DataError::connection_not_found("primary");
        let nest_error: NestError = err.into();
        assert_eq!(nest_error.kind(), NestErrorKind::Data);
        assert_eq!(
            nest_error.code(),
            Some(codes::NEST_DATA_CONNECTION_NOT_FOUND)
        );
    }
}
