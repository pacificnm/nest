#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod apps;
mod docs;
mod launch;
mod nest_root;

#[tauri::command]
fn exit_app() {
    std::process::exit(0);
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            exit_app,
            apps::apps_list,
            launch::apps_resolve_launch,
            launch::apps_spawn,
            docs::docs_list,
            docs::docs_read
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
