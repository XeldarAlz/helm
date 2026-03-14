use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineState {
    pub gdd_exists: bool,
    pub tdd_exists: bool,
    pub workflow_exists: bool,
    pub progress_exists: bool,
    pub current_phase: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetCounts {
    pub scripts: u32,
    pub tests: u32,
    pub prefabs: u32,
    pub configs: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentType {
    Coder,
    Tester,
    Reviewer,
    UnitySetup,
    Committer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Complete,
    Failed,
}
