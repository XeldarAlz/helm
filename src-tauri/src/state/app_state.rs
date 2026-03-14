use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::models::session::SessionId;
use crate::models::settings::AppSettings;
use crate::state::session_state::SessionState;

pub struct AppState {
    pub sessions: Mutex<HashMap<SessionId, SessionState>>,
    pub project_dir: Mutex<Option<PathBuf>>,
    pub settings: Mutex<AppSettings>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            project_dir: Mutex::new(None),
            settings: Mutex::new(AppSettings::default()),
        }
    }
}
