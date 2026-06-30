//! Repository contracts.

use crate::error::DataResult;
use crate::query::ListQuery;

/// Synchronous CRUD repository contract.
pub trait Repository<TEntity, TId>: Send + Sync {
    /// Returns an entity by id.
    fn get(&self, id: TId) -> DataResult<Option<TEntity>>;

    /// Lists entities matching the query.
    fn list(&self, query: ListQuery) -> DataResult<Vec<TEntity>>;

    /// Inserts a new entity.
    fn insert(&self, entity: TEntity) -> DataResult<TEntity>;

    /// Updates an existing entity.
    fn update(&self, entity: TEntity) -> DataResult<TEntity>;

    /// Deletes an entity by id.
    fn delete(&self, id: TId) -> DataResult<()>;
}

#[cfg(feature = "async")]
mod async_repository {
    use async_trait::async_trait;

    use super::*;

    /// Asynchronous CRUD repository contract.
    #[async_trait]
    pub trait AsyncRepository<TEntity, TId>: Send + Sync
    where
        TEntity: Send,
        TId: Send,
    {
        /// Returns an entity by id.
        async fn get(&self, id: TId) -> DataResult<Option<TEntity>>;

        /// Lists entities matching the query.
        async fn list(&self, query: ListQuery) -> DataResult<Vec<TEntity>>;

        /// Inserts a new entity.
        async fn insert(&self, entity: TEntity) -> DataResult<TEntity>;

        /// Updates an existing entity.
        async fn update(&self, entity: TEntity) -> DataResult<TEntity>;

        /// Deletes an entity by id.
        async fn delete(&self, id: TId) -> DataResult<()>;
    }
}

#[cfg(feature = "async")]
pub use async_repository::AsyncRepository;
