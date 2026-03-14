use std::path::PathBuf;

use serde::Serialize;
use tauri::State;
use uuid::Uuid;

use crate::models::session::{
    Message, MessageRole, PipelinePhase, SessionId, SessionStatus, SessionSummary,
};
use crate::process::manager::ProcessManager;
use crate::state::app_state::AppState;
use crate::state::session_state::SessionState;

/// Async-safe wrapper so command signatures stay readable.
pub type ProcessMgr = tokio::sync::Mutex<ProcessManager>;

#[tauri::command]
pub async fn create_session(
    state: State<'_, AppState>,
    process_mgr: State<'_, ProcessMgr>,
    app: tauri::AppHandle,
    phase: PipelinePhase,
) -> Result<SessionId, String> {
    let id = Uuid::new_v4();

    // 1. Create session state
    let session = SessionState::new(id, phase.clone());
    {
        let mut sessions = state.sessions.lock().map_err(|e| e.to_string())?;
        sessions.insert(id, session);
    }

    // 2. Resolve working directory & CLI path from settings
    let (working_dir, cli_path) = {
        let settings = state.settings.lock().map_err(|e| e.to_string())?;
        let dir = if settings.project_dir.is_empty() {
            None
        } else {
            Some(PathBuf::from(&settings.project_dir))
        };
        (dir, settings.claude_cli_path.clone())
    };

    // 3. Register the session (no process spawned yet — each message
    //    spawns its own short-lived `claude --print` process)
    {
        let mut mgr = process_mgr.lock().await;
        mgr.set_cli_path(&cli_path);
        mgr.register_session(id, working_dir)?;

        // 4. Auto-send the phase slash command (if applicable)
        let command = match &phase {
            PipelinePhase::GameIdea => Some("/game-idea"),
            PipelinePhase::Architect => Some("/architect"),
            PipelinePhase::PlanWorkflow => Some("/plan-workflow"),
            PipelinePhase::Orchestrate => Some("/orchestrate"),
            PipelinePhase::Custom => None,
        };
        if let Some(cmd) = command {
            mgr.send(&id, cmd, app).await?;
        }
    }

    Ok(id)
}

#[tauri::command]
pub async fn send_message(
    state: State<'_, AppState>,
    process_mgr: State<'_, ProcessMgr>,
    app: tauri::AppHandle,
    session_id: SessionId,
    message: String,
) -> Result<(), String> {
    // Verify session exists and record the user message
    {
        let mut sessions = state.sessions.lock().map_err(|e| e.to_string())?;
        let session = sessions
            .get_mut(&session_id)
            .ok_or("Session not found")?;

        session.messages.push(Message {
            id: Uuid::new_v4(),
            role: MessageRole::User,
            content: message.clone(),
            timestamp: chrono::Utc::now(),
        });
    }

    // Spawn a new claude --print process for this message
    let mut mgr = process_mgr.lock().await;
    mgr.send(&session_id, &message, app).await
}

#[tauri::command]
pub async fn end_session(
    state: State<'_, AppState>,
    process_mgr: State<'_, ProcessMgr>,
    session_id: SessionId,
) -> Result<(), String> {
    // Kill the child process (ignore errors if already dead)
    {
        let mut mgr = process_mgr.lock().await;
        let _ = mgr.kill(&session_id).await;
    }

    // Mark session as ended
    {
        let mut sessions = state.sessions.lock().map_err(|e| e.to_string())?;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.status = SessionStatus::Ended;
            session.ended_at = Some(chrono::Utc::now());
        } else {
            return Err("Session not found".to_string());
        }
    }

    // Auto-save transcript to disk
    let _ = save_transcript_internal(&state, session_id);

    Ok(())
}

/// Internal helper — saves transcript without needing State wrapper.
fn save_transcript_internal(state: &AppState, session_id: SessionId) -> Result<String, String> {
    let sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    let session = sessions
        .get(&session_id)
        .ok_or("Session not found")?;

    let project_dir = {
        let settings = state.settings.lock().map_err(|e| e.to_string())?;
        if settings.project_dir.is_empty() {
            std::env::current_dir()
                .map_err(|e| e.to_string())?
                .to_string_lossy()
                .to_string()
        } else {
            settings.project_dir.clone()
        }
    };

    let transcripts_dir = PathBuf::from(&project_dir).join("sessions");
    std::fs::create_dir_all(&transcripts_dir).map_err(|e| e.to_string())?;

    let filename = format!("{}_{}.json", session.phase, session_id);
    let filepath = transcripts_dir.join(&filename);

    let transcript = TranscriptFile {
        session_id: session.id,
        phase: session.phase.clone(),
        started_at: session.started_at,
        ended_at: session.ended_at,
        messages: session.messages.clone(),
    };

    let json = serde_json::to_string_pretty(&transcript).map_err(|e| e.to_string())?;
    std::fs::write(&filepath, json).map_err(|e| e.to_string())?;

    Ok(filepath.to_string_lossy().to_string())
}

#[tauri::command]
pub fn list_sessions(state: State<AppState>) -> Result<Vec<SessionSummary>, String> {
    let sessions = state.sessions.lock().map_err(|e| e.to_string())?;

    let mut summaries: Vec<SessionSummary> = sessions
        .values()
        .map(|s| SessionSummary {
            id: s.id,
            phase: s.phase.clone(),
            status: s.status.clone(),
            started_at: s.started_at,
            ended_at: s.ended_at,
            message_count: s.messages.len() as u32,
        })
        .collect();

    // Most recent first
    summaries.sort_by(|a, b| b.started_at.cmp(&a.started_at));
    Ok(summaries)
}

#[tauri::command]
pub fn get_session_messages(
    state: State<AppState>,
    session_id: SessionId,
) -> Result<Vec<Message>, String> {
    let sessions = state.sessions.lock().map_err(|e| e.to_string())?;
    let session = sessions
        .get(&session_id)
        .ok_or("Session not found")?;
    Ok(session.messages.clone())
}

/// Transcript data written to disk.
#[derive(Serialize)]
struct TranscriptFile {
    session_id: SessionId,
    phase: PipelinePhase,
    started_at: chrono::DateTime<chrono::Utc>,
    ended_at: Option<chrono::DateTime<chrono::Utc>>,
    messages: Vec<Message>,
}

#[tauri::command]
pub fn save_transcript(
    state: State<AppState>,
    session_id: SessionId,
) -> Result<String, String> {
    save_transcript_internal(&state, session_id)
}
