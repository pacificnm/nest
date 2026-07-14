//! Blocking Tokio helper for sync module configuration.

use nest_error::{NestError, NestResult};

/// Runs an async future on the current Tokio runtime or a temporary one.
///
/// Used during `Module::configure`, which is synchronous.
///
/// Not yet called anywhere - its consumer, `MqttModule::configure`, lands in
/// a later issue.
#[allow(dead_code)]
pub fn block_on<F, T>(future: F) -> NestResult<T>
where
    F: std::future::Future<Output = NestResult<T>>,
{
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        return tokio::task::block_in_place(|| handle.block_on(future));
    }

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|error| NestError::network(error.to_string()))?;
    runtime.block_on(future)
}
