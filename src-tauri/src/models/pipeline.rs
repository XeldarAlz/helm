use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineState {
    pub gdd_exists: bool,
    pub tdd_exists: bool,
    pub workflow_exists: bool,
    pub project_claude_md_exists: bool,
    pub progress_exists: bool,
    pub catch_up_exists: bool,
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
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Vec<FileNode>,
    pub extension: Option<String>,
    pub size: Option<u64>,
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

impl std::fmt::Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentType::Coder => write!(f, "coder"),
            AgentType::Tester => write!(f, "tester"),
            AgentType::Reviewer => write!(f, "reviewer"),
            AgentType::UnitySetup => write!(f, "unity"),
            AgentType::Committer => write!(f, "committer"),
        }
    }
}

impl AgentType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "coder" => Some(AgentType::Coder),
            "tester" => Some(AgentType::Tester),
            "reviewer" => Some(AgentType::Reviewer),
            "unity" | "unity_setup" | "unity-setup" => Some(AgentType::UnitySetup),
            "committer" | "commit" => Some(AgentType::Committer),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Working,
    Review,
    Done,
    Failed,
}

impl TaskStatus {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "pending" | "queued" => TaskStatus::Pending,
            "working" | "in_progress" | "in-progress" | "active" => TaskStatus::Working,
            "review" | "reviewing" => TaskStatus::Review,
            "done" | "complete" | "completed" | "passed" => TaskStatus::Done,
            "failed" | "error" => TaskStatus::Failed,
            _ => TaskStatus::Pending,
        }
    }
}

// ── Phase 6: Orchestration Models ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrchestrationStatus {
    Idle,
    Running,
    Paused,
    Stopped,
    Completed,
    Failed,
}

impl OrchestrationStatus {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "running" | "active" => OrchestrationStatus::Running,
            "paused" => OrchestrationStatus::Paused,
            "stopped" => OrchestrationStatus::Stopped,
            "completed" | "complete" | "done" => OrchestrationStatus::Completed,
            "failed" | "error" => OrchestrationStatus::Failed,
            _ => OrchestrationStatus::Idle,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationState {
    pub status: OrchestrationStatus,
    pub current_phase: u32,
    pub total_phases: u32,
    pub phase_name: String,
    pub started_at: Option<String>,
    pub phases: Vec<PhaseInfo>,
    pub agents: Vec<AgentInfo>,
    pub tasks: Vec<TaskInfo>,
    pub hooks: Vec<HookResult>,
    pub log: Vec<LogEntry>,
    pub eco_mode: bool,
    pub stop_protected: bool,
    pub last_compact_save: Option<String>,
    pub orchestrator_checkpoint: Option<String>,
}

impl Default for OrchestrationState {
    fn default() -> Self {
        Self {
            status: OrchestrationStatus::Idle,
            current_phase: 0,
            total_phases: 0,
            phase_name: String::new(),
            started_at: None,
            phases: Vec::new(),
            agents: Vec::new(),
            tasks: Vec::new(),
            hooks: Vec::new(),
            log: Vec::new(),
            eco_mode: false,
            stop_protected: false,
            last_compact_save: None,
            orchestrator_checkpoint: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PhaseStatus {
    Done,
    Active,
    Pending,
}

impl PhaseStatus {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "done" | "complete" | "completed" => PhaseStatus::Done,
            "active" | "running" | "in-progress" => PhaseStatus::Active,
            _ => PhaseStatus::Pending,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseInfo {
    pub number: u32,
    pub name: String,
    pub status: PhaseStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Idle,
    Running,
    Reviewing,
    Passed,
    Failed,
}

impl AgentStatus {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "running" | "active" | "working" => AgentStatus::Running,
            "reviewing" | "review" => AgentStatus::Reviewing,
            "passed" | "done" | "complete" => AgentStatus::Passed,
            "failed" | "error" => AgentStatus::Failed,
            _ => AgentStatus::Idle,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub agent_type: AgentType,
    pub status: AgentStatus,
    pub current_task: Option<String>,
    pub progress: u32,
    pub model: Option<String>,
    pub health: AgentHealth,
    pub last_mailbox: Option<MailboxMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentHealth {
    Unknown,
    Alive,
    Stale,
    Dead,
}

impl Default for AgentHealth {
    fn default() -> Self {
        AgentHealth::Unknown
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MailboxMessage {
    pub ts: String,
    #[serde(rename = "type")]
    pub msg_type: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskInfo {
    pub id: String,
    pub title: String,
    pub status: TaskStatus,
    pub agent: Option<String>,
    pub complexity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookStatus {
    Passed,
    Warning,
    Failed,
    Running,
}

impl HookStatus {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "passed" | "pass" | "ok" => HookStatus::Passed,
            "warning" | "warn" => HookStatus::Warning,
            "failed" | "fail" | "error" | "blocked" => HookStatus::Failed,
            "running" | "active" => HookStatus::Running,
            _ => HookStatus::Passed,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult {
    pub name: String,
    pub last_run: Option<String>,
    pub status: HookStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    System,
    Agent,
    Hook,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub source: String,
    pub message: String,
}

// ── Live orchestration state (populated from session text output) ────────────

/// In-memory buffer of orchestration events detected by parsing session
/// text output in real time.  This allows `get_orchestration_state` to
/// return useful data even when the orchestrator agent hasn't written
/// PROGRESS.md or EVENTS.jsonl.
pub struct OrchLiveLog(pub std::sync::Mutex<OrchLiveData>);

#[derive(Debug, Clone, Default)]
pub struct OrchLiveData {
    pub log: Vec<LogEntry>,
    pub status: Option<OrchestrationStatus>,
    pub current_phase: u32,
}

impl OrchLiveLog {
    pub fn new() -> Self {
        Self(std::sync::Mutex::new(OrchLiveData::default()))
    }

    pub fn push(&self, entry: LogEntry) {
        if let Ok(mut data) = self.0.lock() {
            data.log.push(entry);
        }
    }

    pub fn set_running(&self) {
        if let Ok(mut data) = self.0.lock() {
            if data.status.is_none() {
                data.status = Some(OrchestrationStatus::Running);
            }
        }
    }

    pub fn set_phase(&self, phase: u32) {
        if let Ok(mut data) = self.0.lock() {
            data.current_phase = phase;
        }
    }

    pub fn snapshot(&self) -> OrchLiveData {
        self.0.lock().map(|d| d.clone()).unwrap_or_default()
    }

    pub fn clear(&self) {
        if let Ok(mut data) = self.0.lock() {
            *data = OrchLiveData::default();
        }
    }
}
