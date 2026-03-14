pub mod commands;
pub mod models;
pub mod parser;
pub mod process;
pub mod state;
pub mod watcher;

use commands::pipeline::WatcherState;
use commands::session::ProcessMgr;
use commands::settings::load_settings_from_disk;
use process::manager::ProcessManager;
use state::app_state::AppState;
use tauri::Manager;
use watcher::docs::DocsWatcher;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(AppState::new())
        .manage(ProcessMgr::new(ProcessManager::new()))
        .manage(WatcherState(std::sync::Mutex::new(DocsWatcher::new())))
        .setup(|app| {
            let settings = load_settings_from_disk(&app.handle());
            let state = app.state::<AppState>();
            let mut current = state.settings.lock().expect("settings lock poisoned");
            *current = settings;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::session::create_session,
            commands::session::send_message,
            commands::session::end_session,
            commands::session::list_sessions,
            commands::session::get_session_messages,
            commands::session::save_transcript,
            commands::pipeline::get_pipeline_state,
            commands::pipeline::read_document,
            commands::pipeline::get_asset_counts,
            commands::pipeline::check_cli,
            commands::pipeline::start_watching,
            commands::pipeline::stop_watching,
            commands::pipeline::get_file_tree,
            commands::pipeline::read_project_file,
            commands::pipeline::get_orchestration_state,
            commands::pipeline::send_orchestration_command,
            commands::settings::get_settings,
            commands::settings::update_settings,
            commands::git::get_git_log,
            commands::git::get_git_branches,
            commands::git::get_git_status,
            commands::git::get_git_diff,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Helm");
}
