pub mod commands;
pub mod models;
pub mod process;
pub mod state;
pub mod watcher;

use state::app_state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::session::create_session,
            commands::session::send_message,
            commands::session::end_session,
            commands::pipeline::get_pipeline_state,
            commands::pipeline::read_document,
            commands::settings::get_settings,
            commands::settings::update_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Helm");
}
