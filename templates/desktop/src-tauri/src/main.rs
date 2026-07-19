#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use nest_cache::Cache;
use nest_cache_file::{FileCacheAdapter, FileCacheConfig};
use nest_cli_command::CliCommand;
use nest_image::ImageModule;
use nest_tauri::TauriApp;
use nest_theme::ThemeModule;

// This desktop app is a thin client, not where command logic lives — the
// same dispatch (and the same CliCommand variants) exists as a standalone
// binary in ../src-cli/src/main.rs, runnable independently of Tauri. Keep
// the two in sync; do not let real logic accumulate only on this side.
#[tauri::command]
async fn run_cli(command: CliCommand) -> Result<String, String> {
    match command {
        CliCommand::AboutVersion => {
            nest_version::app_version(std::path::Path::new(".")).map_err(|e| e.to_string())
        }
        _ => Err("Unsupported command".into()),
    }
}

fn main() {
    let cache_root = std::env::temp_dir().join("nest-desktop-template-cache");
    let cache = Cache::new(Arc::new(
        FileCacheAdapter::new(FileCacheConfig::new(&cache_root))
            .expect("failed to open image cache directory"),
    ));

    TauriApp::new("nest-desktop-template")
        .module(ThemeModule::default())
        .module(ImageModule::with_cache(cache))
        // `nest-tauri` already attaches the built-in nest_app_metadata /
        // nest_theme_css / image commands to the main invoke_handler before
        // this closure runs. Calling `.invoke_handler(...)` here would
        // replace that registration outright (Tauri's invoke_handler is a
        // setter, not additive) and silently break theme loading — see
        // `TauriApp::with_builder`'s doc comment. App-specific commands must
        // be registered as a Tauri plugin instead; the UI invokes them as
        // `plugin:nest-desktop-template|<command>`.
        .with_builder(|builder| {
            builder.plugin(
                // `R` must be pinned explicitly (`C` then defaults to `()`)
                // — nothing here otherwise fixes `Builder`'s type
                // parameters until `.plugin()` several calls later, which
                // is too late for inference.
                tauri::plugin::Builder::<tauri::Wry>::new("nest-desktop-template")
                    .invoke_handler(tauri::generate_handler![run_cli])
                    .build(),
            )
        })
        .run(tauri::generate_context!());
}
