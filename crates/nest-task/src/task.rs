//! Task trait.

use async_trait::async_trait;
use nest_error::NestResult;

use crate::context::TaskContext;

/// A first-class unit of asynchronous work.
#[async_trait]
pub trait Task: Send + Sync + 'static {
    /// Output produced when the task completes successfully.
    type Output: Send + Sync + 'static;

    /// Returns a stable name for logging, events, and introspection.
    fn name(&self) -> &'static str;

    /// Executes the task.
    async fn run(&self, ctx: TaskContext) -> NestResult<Self::Output>;
}
