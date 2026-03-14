use tauri::State;

use crate::models::settings::AppSettings;
use crate::state::app_state::AppState;

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<AppSettings, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    Ok(settings.clone())
}

#[tauri::command]
pub fn update_settings(
    state: State<AppState>,
    settings: AppSettings,
) -> Result<(), String> {
    let mut current = state.settings.lock().map_err(|e| e.to_string())?;
    *current = settings;
    Ok(())
}
