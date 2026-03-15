use std::fs;
use std::path::PathBuf;

use tauri::{AppHandle, Manager, State};

use crate::models::settings::AppSettings;
use crate::state::app_state::AppState;

/// Resolve the settings file path inside the app data directory.
fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {e}"))?;
    Ok(dir.join("settings.json"))
}

/// Load persisted settings from disk, falling back to defaults.
pub fn load_settings_from_disk(app: &AppHandle) -> AppSettings {
    let path = match settings_path(app) {
        Ok(p) => p,
        Err(_) => return AppSettings::default(),
    };

    match fs::read_to_string(&path) {
        Ok(json) => serde_json::from_str(&json).unwrap_or_default(),
        Err(_) => AppSettings::default(),
    }
}

/// Atomically write settings to disk (.tmp → rename).
fn save_settings_to_disk(app: &AppHandle, settings: &AppSettings) -> Result<(), String> {
    let path = settings_path(app)?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create settings dir: {e}"))?;
    }

    let json =
        serde_json::to_string_pretty(settings).map_err(|e| format!("Serialize error: {e}"))?;

    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, &json).map_err(|e| format!("Write error: {e}"))?;
    fs::rename(&tmp, &path).map_err(|e| format!("Rename error: {e}"))?;

    Ok(())
}

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Result<AppSettings, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    Ok(settings.clone())
}

#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    state: State<AppState>,
    settings: AppSettings,
) -> Result<(), String> {
    // Sync project_dir
    {
        let mut project_dir = state.project_dir.lock().map_err(|e| e.to_string())?;
        *project_dir = if settings.project_dir.is_empty() {
            None
        } else {
            Some(std::path::PathBuf::from(&settings.project_dir))
        };
    }

    let mut current = state.settings.lock().map_err(|e| e.to_string())?;
    *current = settings.clone();
    save_settings_to_disk(&app, &settings)?;
    Ok(())
}
