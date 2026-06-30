//! Panic hook integration with tracing.

#![allow(clippy::incompatible_msrv)]

use std::panic::{self, PanicHookInfo};
use std::sync::OnceLock;

type PanicHook = Box<dyn Fn(&PanicHookInfo<'_>) + Send + Sync>;

static PREVIOUS_HOOK: OnceLock<PanicHook> = OnceLock::new();

/// Installs a panic hook that logs panics via tracing before delegating to the previous hook.
pub fn install_panic_hook() {
    let previous = panic::take_hook();
    let _ = PREVIOUS_HOOK.set(previous);

    panic::set_hook(Box::new(|info| {
        tracing::error!(
            target: "nest_panic",
            message = %info,
            location = ?info.location(),
            "panic occurred"
        );

        if let Some(previous) = PREVIOUS_HOOK.get() {
            previous(info);
        }
    }));
}
