use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub project_dir: String,
    pub claude_cli_path: String,
    pub max_concurrent_agents: u8,
    pub agent_models: AgentModelConfig,
    pub font_size: u8,
    pub sidebar_collapsed: bool,
    #[serde(default)]
    pub theme: String,
    #[serde(default)]
    pub reduced_motion: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentModelConfig {
    pub coder: String,
    pub tester: String,
    pub reviewer: String,
    pub unity_setup: String,
    pub committer: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            project_dir: String::new(),
            claude_cli_path: "claude".to_string(),
            max_concurrent_agents: 3,
            agent_models: AgentModelConfig::default(),
            font_size: 13,
            sidebar_collapsed: false,
            theme: "dark".to_string(),
            reduced_motion: false,
        }
    }
}

impl Default for AgentModelConfig {
    fn default() -> Self {
        Self {
            coder: "opus".to_string(),
            tester: "sonnet".to_string(),
            reviewer: "opus".to_string(),
            unity_setup: "sonnet".to_string(),
            committer: "haiku".to_string(),
        }
    }
}
