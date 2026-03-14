use tauri::State;
use uuid::Uuid;

use crate::models::session::{PipelinePhase, SessionId, SessionStatus};
use crate::state::app_state::AppState;
use crate::state::session_state::SessionState;

#[tauri::command]
pub fn create_session(
    state: State<AppState>,
    phase: PipelinePhase,
) -> Result<SessionId, String> {
    let id = Uuid::new_v4();
    let session = SessionState::new(id, phase);
    let mut sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    sessions.insert(id, session);
    Ok(id)
}

#[tauri::command]
pub fn send_message(
    state: State<AppState>,
    session_id: SessionId,
    _message: String,
) -> Result<(), String> {
    let sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    if !sessions.contains_key(&session_id) {
        return Err("Session not found".to_string());
    }
    // TODO: Write to child process stdin when ProcessManager is implemented
    Ok(())
}

#[tauri::command]
pub fn end_session(
    state: State<AppState>,
    session_id: SessionId,
) -> Result<(), String> {
    let mut sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    if let Some(session) = sessions.get_mut(&session_id) {
        session.status = SessionStatus::Ended;
        session.ended_at = Some(chrono::Utc::now());
        Ok(())
    } else {
        Err("Session not found".to_string())
    }
}
