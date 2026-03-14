use crate::models::session::{Message, PipelinePhase, SessionId, SessionStatus};
use chrono::{DateTime, Utc};

pub struct SessionState {
    pub id: SessionId,
    pub phase: PipelinePhase,
    pub status: SessionStatus,
    pub messages: Vec<Message>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

impl SessionState {
    pub fn new(id: SessionId, phase: PipelinePhase) -> Self {
        Self {
            id,
            phase,
            status: SessionStatus::Active,
            messages: Vec::new(),
            started_at: Utc::now(),
            ended_at: None,
        }
    }
}
