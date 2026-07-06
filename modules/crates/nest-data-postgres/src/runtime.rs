//! Blocking Tokio helper for sync module configuration.

use nest_data::{DataError, DataResult};

/// Runs an async future on the current Tokio runtime or a temporary one.
///
/// Used during [`nest_core::Module::configure`] which is synchronous.
pub fn block_on<F, T>(future: F) -> DataResult<T>
where
    F: std::future::Future<Output = DataResult<T>>,
{
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        return tokio::task::block_in_place(|| handle.block_on(future));
    }

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|error| DataError::connection_error(error.to_string()))?;
    runtime.block_on(future)
}
