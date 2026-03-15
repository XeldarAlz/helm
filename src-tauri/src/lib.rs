pub mod commands;
pub mod models;
pub mod parser;
pub mod process;
pub mod state;
pub mod watcher;

use std::path::PathBuf;

use commands::pipeline::WatcherState;
use commands::session::ProcessMgr;
use commands::settings::load_settings_from_disk;
use process::manager::ProcessManager;
use state::app_state::AppState;
use tauri::Manager;
use watcher::docs::DocsWatcher;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Install a panic hook so crashes are visible in the terminal
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        eprintln!("\n[helm:PANIC] {info}");
        if let Some(loc) = info.location() {
            eprintln!("[helm:PANIC] at {}:{}:{}", loc.file(), loc.line(), loc.column());
        }
        default_hook(info);
    }));

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(AppState::new())
        .manage(ProcessMgr::new(ProcessManager::new()))
        .manage(WatcherState(std::sync::Mutex::new(DocsWatcher::new())))
        .setup(|app| {
            let settings = load_settings_from_disk(&app.handle());
            let state = app.state::<AppState>();

            // Sync project_dir from settings
            if !settings.project_dir.is_empty() {
                let mut project_dir = state.project_dir.lock().expect("project_dir lock poisoned");
                *project_dir = Some(PathBuf::from(&settings.project_dir));
            }

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
        .build(tauri::generate_context!())
        .expect("error while building Helm");

    app.run(|app_handle, event| {
        if let tauri::RunEvent::Exit = event {
            // Send SIGTERM to all child processes so Claude Code can clean up
            // gracefully.  We use try_lock because the async runtime may
            // already be winding down.
            let state = app_handle.state::<ProcessMgr>();
            if let Ok(mut guard) = state.try_lock() {
                guard.kill_all_sync();
            };
        }
    });
}
