use serde::{Deserialize, Serialize};

use crate::models::pipeline::AgentType;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClaudeEvent {
    TextOutput { text: String },
    Question { text: String, suggested_answers: Vec<String> },
    AgentSpawned { agent_type: AgentType, task: String },
    AgentCompleted { agent_type: AgentType, result: String },
    FileChanged { path: String, action: String },
    PhaseChange { phase: u32, name: String },
    Streaming,
    ResponseComplete,
}

pub struct OutputParser {
    buffer: String,
}

impl OutputParser {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
        }
    }

    pub fn parse(&mut self, chunk: &str) -> Vec<ClaudeEvent> {
        self.buffer.push_str(chunk);
        // TODO: Implement parsing logic in Phase 2
        let events = vec![ClaudeEvent::TextOutput {
            text: self.buffer.clone(),
        }];
        self.buffer.clear();
        events
    }
}
