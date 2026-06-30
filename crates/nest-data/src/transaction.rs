//! Transaction contracts.

use crate::error::DataResult;

/// A synchronous database transaction.
pub trait Transaction: Send {
    /// Commits the transaction.
    fn commit(self: Box<Self>) -> DataResult<()>;

    /// Rolls back the transaction.
    fn rollback(self: Box<Self>) -> DataResult<()>;
}

/// Types that can begin synchronous transactions.
pub trait Transactional: Send + Sync {
    /// Begins a new transaction.
    fn begin(&self) -> DataResult<Box<dyn Transaction>>;
}

#[cfg(feature = "async")]
mod async_transaction {
    use async_trait::async_trait;

    use super::*;

    /// An asynchronous database transaction.
    #[async_trait]
    pub trait AsyncTransaction: Send {
        /// Commits the transaction.
        async fn commit(self: Box<Self>) -> DataResult<()>;

        /// Rolls back the transaction.
        async fn rollback(self: Box<Self>) -> DataResult<()>;
    }

    /// Types that can begin asynchronous transactions.
    #[async_trait]
    pub trait AsyncTransactional: Send + Sync {
        /// Begins a new transaction.
        async fn begin(&self) -> DataResult<Box<dyn AsyncTransaction>>;
    }
}

#[cfg(feature = "async")]
pub use async_transaction::{AsyncTransaction, AsyncTransactional};
