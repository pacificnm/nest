#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod accounts;
mod agent;
mod agent_settings;
mod apps;
mod claude;
mod claude_settings;
mod config_host;
mod docs;
mod files;
mod launch;
mod nest_root;
mod ollama;
mod terminal;
mod themes;

use agent::AgentPty;
use agent_settings::AgentSettingsStore;
use claude_settings::ClaudeSettingsStore;
use terminal::TerminalManager;

#[tauri::command]
fn exit_app() {
    std::process::exit(0);
}

// Terminal commands
#[tauri::command]
fn terminal_open(
    app: tauri::AppHandle,
    manager: tauri::State<terminal::TerminalManager>,
    id: String,
    cwd: Option<String>,
    shell: Option<String>,
    rows: u16,
    cols: u16,
) -> Result<(), String> {
    let cwd = cwd.filter(|s| !s.is_empty()).map(std::path::PathBuf::from);
    let shell = shell.filter(|s| !s.is_empty());
    manager
        .open(app, id, cwd, shell.as_deref(), rows, cols)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn terminal_input(
    manager: tauri::State<terminal::TerminalManager>,
    id: String,
    data: String,
) -> Result<(), String> {
    manager.input(&id, &data).map_err(|e| e.to_string())
}

#[tauri::command]
fn terminal_resize(
    manager: tauri::State<terminal::TerminalManager>,
    id: String,
    rows: u16,
    cols: u16,
) -> Result<(), String> {
    manager.resize(&id, rows, cols).map_err(|e| e.to_string())
}

#[tauri::command]
fn terminal_close(
    manager: tauri::State<terminal::TerminalManager>,
    id: String,
) -> Result<(), String> {
    manager.close(&id);
    Ok(())
}

#[tauri::command]
fn terminal_list(manager: tauri::State<terminal::TerminalManager>) -> Result<Vec<String>, String> {
    Ok(manager.list())
}

// Agent commands
#[tauri::command]
#[allow(clippy::too_many_arguments)]
fn agent_launch(
    app: tauri::AppHandle,
    manager: tauri::State<AgentPty>,
    runtime: String,
    model: String,
    ollama_host: Option<String>,
    cwd: Option<String>,
    direct: bool,
    rows: u16,
    cols: u16,
) -> Result<(), String> {
    let cwd = cwd.filter(|s| !s.is_empty()).map(std::path::PathBuf::from);
    let ollama_host = ollama_host.filter(|s| !s.is_empty());
    manager.launch(
        app,
        &runtime,
        &model,
        ollama_host.as_deref(),
        cwd,
        direct,
        rows,
        cols,
    )
}

#[tauri::command]
fn agent_input(manager: tauri::State<AgentPty>, data: String) -> Result<(), String> {
    manager.input(&data)
}

#[tauri::command]
fn agent_resize(manager: tauri::State<AgentPty>, rows: u16, cols: u16) -> Result<(), String> {
    manager.resize(rows, cols)
}

#[tauri::command]
fn agent_stop(manager: tauri::State<AgentPty>) -> Result<(), String> {
    manager.stop();
    Ok(())
}

#[tauri::command]
fn agent_status(manager: tauri::State<AgentPty>) -> Result<bool, String> {
    Ok(manager.is_running())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(TerminalManager::default())
        .manage(AgentPty::default())
        .manage(AgentSettingsStore::load(config_host::resolve_config_path()))
        .manage(ClaudeSettingsStore::load(config_host::resolve_config_path()))
        .invoke_handler(tauri::generate_handler![
            exit_app,
            apps::apps_list,
            launch::apps_resolve_launch,
            launch::apps_spawn,
            launch::apps_launch_kiwi,
            docs::docs_list,
            docs::docs_read,
            files::files_info,
            files::files_list,
            files::files_read_text,
            files::files_write_text,
            files::files_create_file,
            files::files_create_dir,
            files::files_rename,
            files::files_delete,
            files::files_copy,
            files::files_reveal,
            terminal_open,
            terminal_input,
            terminal_resize,
            terminal_close,
            terminal_list,
            agent_launch,
            agent_input,
            agent_resize,
            agent_stop,
            agent_status,
            agent_settings::agent_settings_get,
            agent_settings::agent_settings_save,
            ollama::ollama_list_models,
            ollama::ollama_auth_status,
            ollama::ollama_signin,
            ollama::ollama_signout,
            accounts::codex_account_status,
            accounts::codex_login,
            accounts::codex_logout,
            themes::themes_list,
            claude_settings::claude_settings_get,
            claude_settings::claude_settings_save,
            claude::claude_list_skills,
            claude::claude_list_agents,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
