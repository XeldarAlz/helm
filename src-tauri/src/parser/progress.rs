use crate::models::pipeline::{
    AgentHealth, AgentInfo, AgentStatus, AgentType, HookResult, HookStatus, LogEntry, LogLevel,
    OrchestrationState, OrchestrationStatus, PhaseInfo, PhaseStatus, TaskInfo, TaskStatus,
};

/// Parse a PROGRESS.md file into structured OrchestrationState.
///
/// Expected format:
/// ```markdown
/// # Orchestration Progress
/// ## Status: running
/// ## Phase: 3 / 7
/// ## Phase Name: Pure C# Logic Systems
/// ## Started: 2026-03-14T10:00:00Z
///
/// ## Agents
/// | Agent | Type | Status | Task | Progress |
/// |-------|------|--------|------|----------|
/// | coder-1 | coder | running | Implement PlayerMovement | 75% |
///
/// ## Tasks
/// | ID | Title | Status | Agent | Complexity |
/// |----|-------|--------|-------|------------|
/// | 3.1 | Implement PlayerMovement | working | coder-1 | M |
///
/// ## Phases
/// | # | Name | Status |
/// |---|------|--------|
/// | 1 | Project Setup | done |
///
/// ## Hooks
/// | Hook | Last Run | Result |
/// |------|----------|--------|
/// | check-pure-csharp | 2026-03-14T10:15:00Z | passed |
///
/// ## Log
/// [2026-03-14T10:00:00Z] [system] Orchestration started
/// ```
pub fn parse_progress(content: &str) -> OrchestrationState {
    let mut state = OrchestrationState::default();

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Header metadata
        if line.starts_with("## Status:") {
            let val = line.strip_prefix("## Status:").unwrap_or("").trim();
            state.status = OrchestrationStatus::from_str(val);
        } else if line.starts_with("## Phase:") {
            let val = line.strip_prefix("## Phase:").unwrap_or("").trim();
            if let Some((current, total)) = val.split_once('/') {
                state.current_phase = current.trim().parse().unwrap_or(0);
                state.total_phases = total.trim().parse().unwrap_or(0);
            }
        } else if line.starts_with("## Phase Name:") {
            state.phase_name = line
                .strip_prefix("## Phase Name:")
                .unwrap_or("")
                .trim()
                .to_string();
        } else if line.starts_with("## Started:") {
            let val = line.strip_prefix("## Started:").unwrap_or("").trim();
            if !val.is_empty() {
                state.started_at = Some(val.to_string());
            }
        }
        // Section tables
        else if line == "## Agents" {
            i += 1;
            state.agents = parse_agents_table(&lines, &mut i);
            continue;
        } else if line == "## Tasks" {
            i += 1;
            state.tasks = parse_tasks_table(&lines, &mut i);
            continue;
        } else if line == "## Phases" {
            i += 1;
            state.phases = parse_phases_table(&lines, &mut i);
            continue;
        } else if line == "## Hooks" {
            i += 1;
            state.hooks = parse_hooks_table(&lines, &mut i);
            continue;
        } else if line == "## Log" {
            i += 1;
            state.log = parse_log_entries(&lines, &mut i);
            continue;
        }

        i += 1;
    }

    // Extract model info from log entries (format: "... model: opus" or "... (complexity: XL, model: opus)")
    for entry in &state.log {
        if let Some(model_pos) = entry.message.find("model: ") {
            let model_str = &entry.message[model_pos + 7..];
            let model = model_str
                .split(|c: char| c == ')' || c == ',' || c.is_whitespace())
                .next()
                .unwrap_or("")
                .to_string();
            if !model.is_empty() {
                // Match to agent by source (e.g., "agent:coder-1")
                if let Some(agent_id) = entry.source.strip_prefix("agent:") {
                    if let Some(agent) = state.agents.iter_mut().find(|a| a.id == agent_id) {
                        agent.model = Some(model);
                    }
                }
            }
        }
    }

    // Detect eco mode from log entries
    state.eco_mode = state.log.iter().any(|entry| {
        entry.message.contains("[eco]")
            || entry
                .message
                .to_lowercase()
                .contains("eco mode active")
    });

    state
}

/// Parse markdown table rows, skipping the header and separator rows.
/// Returns rows as Vec<Vec<String>> where each inner vec is the cells of a row.
fn parse_table_rows(lines: &[&str], i: &mut usize) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    let mut header_seen = false;
    let mut separator_seen = false;

    while *i < lines.len() {
        let line = lines[*i].trim();

        // End of table
        if line.is_empty() || (line.starts_with('#') && !line.starts_with('|')) {
            break;
        }

        if !line.starts_with('|') {
            *i += 1;
            continue;
        }

        if !header_seen {
            header_seen = true;
            *i += 1;
            continue;
        }

        if !separator_seen {
            // Separator row like |---|---|
            if line.contains("---") {
                separator_seen = true;
                *i += 1;
                continue;
            }
        }

        // Data row
        let cells: Vec<String> = line
            .split('|')
            .filter(|c| !c.trim().is_empty())
            .map(|c| c.trim().to_string())
            .collect();

        if !cells.is_empty() {
            rows.push(cells);
        }

        *i += 1;
    }

    rows
}

fn parse_agents_table(lines: &[&str], i: &mut usize) -> Vec<AgentInfo> {
    let rows = parse_table_rows(lines, i);
    rows.into_iter()
        .filter_map(|cells| {
            if cells.len() < 4 {
                return None;
            }
            let agent_type = AgentType::from_str(&cells[1])?;
            Some(AgentInfo {
                id: cells[0].clone(),
                agent_type,
                status: AgentStatus::from_str(&cells[2]),
                current_task: if cells.len() > 3 && cells[3] != "—" && cells[3] != "-" {
                    Some(cells[3].clone())
                } else {
                    None
                },
                progress: if cells.len() > 4 {
                    cells[4].trim_end_matches('%').parse().unwrap_or(0)
                } else {
                    0
                },
                model: None,
                health: AgentHealth::Unknown,
                last_mailbox: None,
            })
        })
        .collect()
}

fn parse_tasks_table(lines: &[&str], i: &mut usize) -> Vec<TaskInfo> {
    let rows = parse_table_rows(lines, i);
    rows.into_iter()
        .filter_map(|cells| {
            if cells.len() < 5 {
                return None;
            }
            Some(TaskInfo {
                id: cells[0].clone(),
                title: cells[1].clone(),
                status: TaskStatus::from_str(&cells[2]),
                agent: if cells[3] != "—" && cells[3] != "-" {
                    Some(cells[3].clone())
                } else {
                    None
                },
                complexity: cells[4].clone(),
            })
        })
        .collect()
}

fn parse_phases_table(lines: &[&str], i: &mut usize) -> Vec<PhaseInfo> {
    let rows = parse_table_rows(lines, i);
    rows.into_iter()
        .filter_map(|cells| {
            if cells.len() < 3 {
                return None;
            }
            Some(PhaseInfo {
                number: cells[0].parse().unwrap_or(0),
                name: cells[1].clone(),
                status: PhaseStatus::from_str(&cells[2]),
            })
        })
        .collect()
}

fn parse_hooks_table(lines: &[&str], i: &mut usize) -> Vec<HookResult> {
    let rows = parse_table_rows(lines, i);
    rows.into_iter()
        .filter_map(|cells| {
            if cells.len() < 3 {
                return None;
            }
            Some(HookResult {
                name: cells[0].clone(),
                last_run: if cells[1] != "—" && cells[1] != "-" {
                    Some(cells[1].clone())
                } else {
                    None
                },
                status: HookStatus::from_str(&cells[2]),
            })
        })
        .collect()
}

fn parse_log_entries(lines: &[&str], i: &mut usize) -> Vec<LogEntry> {
    let mut entries = Vec::new();

    while *i < lines.len() {
        let line = lines[*i].trim();

        if line.is_empty() {
            *i += 1;
            continue;
        }

        // Stop at next section
        if line.starts_with("## ") {
            break;
        }

        // Format: [timestamp] [source] message
        if line.starts_with('[') {
            if let Some(entry) = parse_log_line(line) {
                entries.push(entry);
            }
        }

        *i += 1;
    }

    entries
}

fn parse_log_line(line: &str) -> Option<LogEntry> {
    // [2026-03-14T10:00:00Z] [system] Orchestration started
    let rest = line.strip_prefix('[')?;
    let (timestamp, rest) = rest.split_once(']')?;
    let rest = rest.trim();
    let rest = rest.strip_prefix('[')?;
    let (source, message) = rest.split_once(']')?;
    let message = message.trim();

    let source = source.trim();
    let level = if source == "system" {
        LogLevel::System
    } else if source.starts_with("agent") {
        LogLevel::Agent
    } else if source == "hook" || source.starts_with("hook") {
        LogLevel::Hook
    } else if source == "error" {
        LogLevel::Error
    } else {
        LogLevel::System
    };

    Some(LogEntry {
        timestamp: timestamp.trim().to_string(),
        level,
        source: source.to_string(),
        message: message.to_string(),
    })
}

/// Parse a `docs/EVENTS.jsonl` file to derive orchestration state when
/// PROGRESS.md is missing or empty.  Each line is a JSON object with
/// `ts`, `event`, and `data` fields.
pub fn parse_events_jsonl(content: &str) -> OrchestrationState {
    let mut state = OrchestrationState::default();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let v: serde_json::Value = match serde_json::from_str(line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let ts = v
            .get("ts")
            .and_then(|t| t.as_str())
            .unwrap_or("")
            .to_string();
        let event_type = match v.get("event").and_then(|e| e.as_str()) {
            Some(e) => e,
            None => continue,
        };
        let data = v.get("data");

        match event_type {
            "orchestration_started" => {
                state.status = OrchestrationStatus::Running;
                if state.started_at.is_none() && !ts.is_empty() {
                    state.started_at = Some(ts.clone());
                }
                let total = data
                    .and_then(|d| d.get("phases"))
                    .and_then(|p| p.as_u64())
                    .unwrap_or(0) as u32;
                if total > 0 {
                    state.total_phases = total;
                }
                state.log.push(LogEntry {
                    timestamp: ts,
                    level: LogLevel::System,
                    source: "system".to_string(),
                    message: "Orchestration started".to_string(),
                });
            }
            "agent_spawned" => {
                let agent = data
                    .and_then(|d| d.get("agent"))
                    .and_then(|a| a.as_str())
                    .unwrap_or("unknown");
                let task = data
                    .and_then(|d| d.get("task"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("");
                let model = data
                    .and_then(|d| d.get("model"))
                    .and_then(|m| m.as_str())
                    .unwrap_or("");
                let agent_type_str = data
                    .and_then(|d| d.get("type"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("coder");
                let phase = data
                    .and_then(|d| d.get("phase"))
                    .and_then(|p| p.as_u64())
                    .unwrap_or(0) as u32;

                if phase > 0 {
                    state.current_phase = phase;
                }

                // Add or update agent in the agents list
                if let Some(atype) = AgentType::from_str(agent_type_str) {
                    if !state.agents.iter().any(|a| a.id == agent) {
                        state.agents.push(AgentInfo {
                            id: agent.to_string(),
                            agent_type: atype,
                            status: AgentStatus::Running,
                            current_task: Some(task.to_string()),
                            progress: 0,
                            model: if model.is_empty() {
                                None
                            } else {
                                Some(model.to_string())
                            },
                            health: AgentHealth::Unknown,
                            last_mailbox: None,
                        });
                    } else if let Some(a) = state.agents.iter_mut().find(|a| a.id == agent) {
                        a.status = AgentStatus::Running;
                        a.current_task = Some(task.to_string());
                    }
                }

                state.log.push(LogEntry {
                    timestamp: ts,
                    level: LogLevel::Agent,
                    source: format!("agent:{}", agent),
                    message: format!(
                        "Spawned for task {}{}",
                        task,
                        if model.is_empty() {
                            String::new()
                        } else {
                            format!(" (model: {})", model)
                        }
                    ),
                });
            }
            "agent_completed" => {
                let agent = data
                    .and_then(|d| d.get("agent"))
                    .and_then(|a| a.as_str())
                    .unwrap_or("unknown");
                let task = data
                    .and_then(|d| d.get("task"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("");

                if let Some(a) = state.agents.iter_mut().find(|a| a.id == agent) {
                    a.status = AgentStatus::Passed;
                }

                state.log.push(LogEntry {
                    timestamp: ts,
                    level: LogLevel::Agent,
                    source: format!("agent:{}", agent),
                    message: format!("Completed task {}", task),
                });
            }
            "agent_failed" => {
                let agent = data
                    .and_then(|d| d.get("agent"))
                    .and_then(|a| a.as_str())
                    .unwrap_or("unknown");
                let reason = data
                    .and_then(|d| d.get("reason"))
                    .and_then(|r| r.as_str())
                    .unwrap_or("");

                if let Some(a) = state.agents.iter_mut().find(|a| a.id == agent) {
                    a.status = AgentStatus::Failed;
                }

                state.log.push(LogEntry {
                    timestamp: ts,
                    level: LogLevel::Error,
                    source: format!("agent:{}", agent),
                    message: format!("Failed: {}", reason),
                });
            }
            "review_verdict" => {
                let verdict = data
                    .and_then(|d| d.get("verdict"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let reviewer = data
                    .and_then(|d| d.get("reviewer"))
                    .and_then(|r| r.as_str())
                    .unwrap_or("reviewer");

                state.log.push(LogEntry {
                    timestamp: ts,
                    level: LogLevel::Agent,
                    source: format!("agent:{}", reviewer),
                    message: format!("Review verdict: {}", verdict),
                });
            }
            "phase_transition" => {
                let to = data
                    .and_then(|d| d.get("to"))
                    .and_then(|t| t.as_u64())
                    .unwrap_or(0) as u32;
                let name = data
                    .and_then(|d| d.get("name"))
                    .and_then(|n| n.as_str())
                    .unwrap_or("");

                if to > 0 {
                    state.current_phase = to;
                    state.phase_name = name.to_string();
                }

                state.log.push(LogEntry {
                    timestamp: ts,
                    level: LogLevel::System,
                    source: "system".to_string(),
                    message: format!("Phase {}: {}", to, name),
                });
            }
            "commit_created" => {
                let sha = data
                    .and_then(|d| d.get("sha"))
                    .and_then(|s| s.as_str())
                    .unwrap_or("");
                let message = data
                    .and_then(|d| d.get("message"))
                    .and_then(|m| m.as_str())
                    .unwrap_or("");

                state.log.push(LogEntry {
                    timestamp: ts,
                    level: LogLevel::System,
                    source: "system".to_string(),
                    message: format!("Commit {}: {}", &sha[..sha.len().min(7)], message),
                });
            }
            "orchestration_paused" => {
                state.status = OrchestrationStatus::Paused;
                state.log.push(LogEntry {
                    timestamp: ts,
                    level: LogLevel::System,
                    source: "system".to_string(),
                    message: "Orchestration paused".to_string(),
                });
            }
            "orchestration_completed" => {
                state.status = OrchestrationStatus::Completed;
                state.log.push(LogEntry {
                    timestamp: ts,
                    level: LogLevel::System,
                    source: "system".to_string(),
                    message: "Orchestration completed".to_string(),
                });
            }
            "orchestration_failed" => {
                state.status = OrchestrationStatus::Failed;
                let reason = data
                    .and_then(|d| d.get("reason"))
                    .and_then(|r| r.as_str())
                    .unwrap_or("Unknown error");
                state.log.push(LogEntry {
                    timestamp: ts,
                    level: LogLevel::Error,
                    source: "system".to_string(),
                    message: format!("Orchestration failed: {}", reason),
                });
            }
            "task_status" => {
                let task = data
                    .and_then(|d| d.get("task"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("");
                let to = data
                    .and_then(|d| d.get("to"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("");
                let agent = data
                    .and_then(|d| d.get("agent"))
                    .and_then(|a| a.as_str())
                    .unwrap_or("");

                state.log.push(LogEntry {
                    timestamp: ts,
                    level: LogLevel::Agent,
                    source: if agent.is_empty() {
                        "system".to_string()
                    } else {
                        format!("agent:{}", agent)
                    },
                    message: format!("Task {} → {}", task, to),
                });
            }
            "error" | "blocker" => {
                let message = data
                    .and_then(|d| d.get("message"))
                    .and_then(|m| m.as_str())
                    .unwrap_or("");
                let agent = data
                    .and_then(|d| d.get("agent"))
                    .and_then(|a| a.as_str())
                    .unwrap_or("system");

                state.log.push(LogEntry {
                    timestamp: ts,
                    level: LogLevel::Error,
                    source: if agent == "system" {
                        "system".to_string()
                    } else {
                        format!("agent:{}", agent)
                    },
                    message: format!(
                        "{}{}",
                        if event_type == "blocker" {
                            "BLOCKER: "
                        } else {
                            ""
                        },
                        message
                    ),
                });
            }
            _ => {
                // Unknown event type — still add to log for visibility
                state.log.push(LogEntry {
                    timestamp: ts,
                    level: LogLevel::System,
                    source: "system".to_string(),
                    message: format!("[{}] {:?}", event_type, data),
                });
            }
        }
    }

    state
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_PROGRESS: &str = r#"# Orchestration Progress
## Status: running
## Phase: 3 / 7
## Phase Name: Pure C# Logic Systems
## Started: 2026-03-14T10:00:00Z

## Agents
| Agent | Type | Status | Task | Progress |
|-------|------|--------|------|----------|
| coder-1 | coder | running | Implement PlayerMovement | 75% |
| coder-2 | coder | running | Implement CombatSystem | 30% |
| tester-1 | tester | idle | — | 0% |
| reviewer-1 | reviewer | reviewing | Review InputSystem | 0% |
| committer-1 | committer | idle | — | 0% |

## Tasks
| ID | Title | Status | Agent | Complexity |
|----|-------|--------|-------|------------|
| 3.1 | Implement PlayerMovement | working | coder-1 | M |
| 3.2 | Implement CombatSystem | working | coder-2 | L |
| 3.3 | Implement InputSystem | review | reviewer-1 | S |
| 3.4 | Test PlayerMovement | pending | — | M |
| 3.5 | Test CombatSystem | pending | — | L |

## Phases
| # | Name | Status |
|---|------|--------|
| 1 | Project Setup | done |
| 2 | ScriptableObject Configs | done |
| 3 | Pure C# Logic Systems | active |
| 4 | MonoBehaviour Wrappers | pending |
| 5 | Unity Scene Setup | pending |
| 6 | Integration Testing | pending |
| 7 | Polish & Build | pending |

## Hooks
| Hook | Last Run | Result |
|------|----------|--------|
| check-pure-csharp | 2026-03-14T10:15:00Z | passed |
| check-naming-conventions | 2026-03-14T10:15:00Z | warning |
| check-compile | 2026-03-14T10:14:00Z | passed |

## Log
[2026-03-14T10:00:00Z] [system] Orchestration started
[2026-03-14T10:00:01Z] [system] Phase 1: Project Setup — starting
[2026-03-14T10:02:00Z] [agent:coder-1] Implementing project structure
[2026-03-14T10:05:00Z] [system] Phase 1: Project Setup — complete
[2026-03-14T10:05:01Z] [system] Phase 2: ScriptableObject Configs — starting
[2026-03-14T10:10:00Z] [hook:check-pure-csharp] Passed validation
[2026-03-14T10:12:00Z] [error] Build warning in CombatSystem.cs
"#;

    #[test]
    fn parses_status_metadata() {
        let state = parse_progress(SAMPLE_PROGRESS);
        assert!(matches!(state.status, OrchestrationStatus::Running));
        assert_eq!(state.current_phase, 3);
        assert_eq!(state.total_phases, 7);
        assert_eq!(state.phase_name, "Pure C# Logic Systems");
        assert_eq!(state.started_at, Some("2026-03-14T10:00:00Z".to_string()));
    }

    #[test]
    fn parses_agents() {
        let state = parse_progress(SAMPLE_PROGRESS);
        assert_eq!(state.agents.len(), 5);
        assert_eq!(state.agents[0].id, "coder-1");
        assert!(matches!(state.agents[0].agent_type, AgentType::Coder));
        assert!(matches!(state.agents[0].status, AgentStatus::Running));
        assert_eq!(
            state.agents[0].current_task,
            Some("Implement PlayerMovement".to_string())
        );
        assert_eq!(state.agents[0].progress, 75);
        assert!(matches!(state.agents[2].status, AgentStatus::Idle));
        assert!(state.agents[2].current_task.is_none());
    }

    #[test]
    fn parses_tasks() {
        let state = parse_progress(SAMPLE_PROGRESS);
        assert_eq!(state.tasks.len(), 5);
        assert_eq!(state.tasks[0].id, "3.1");
        assert_eq!(state.tasks[0].title, "Implement PlayerMovement");
        assert!(matches!(state.tasks[0].status, TaskStatus::Working));
        assert_eq!(state.tasks[0].agent, Some("coder-1".to_string()));
        assert_eq!(state.tasks[0].complexity, "M");
        assert!(matches!(state.tasks[3].status, TaskStatus::Pending));
        assert!(state.tasks[3].agent.is_none());
    }

    #[test]
    fn parses_phases() {
        let state = parse_progress(SAMPLE_PROGRESS);
        assert_eq!(state.phases.len(), 7);
        assert_eq!(state.phases[0].number, 1);
        assert_eq!(state.phases[0].name, "Project Setup");
        assert!(matches!(state.phases[0].status, PhaseStatus::Done));
        assert!(matches!(state.phases[2].status, PhaseStatus::Active));
        assert!(matches!(state.phases[3].status, PhaseStatus::Pending));
    }

    #[test]
    fn parses_hooks() {
        let state = parse_progress(SAMPLE_PROGRESS);
        assert_eq!(state.hooks.len(), 3);
        assert_eq!(state.hooks[0].name, "check-pure-csharp");
        assert!(matches!(state.hooks[0].status, HookStatus::Passed));
        assert!(matches!(state.hooks[1].status, HookStatus::Warning));
    }

    #[test]
    fn parses_log() {
        let state = parse_progress(SAMPLE_PROGRESS);
        assert_eq!(state.log.len(), 7);
        assert_eq!(state.log[0].source, "system");
        assert!(matches!(state.log[0].level, LogLevel::System));
        assert_eq!(state.log[0].message, "Orchestration started");
        assert!(matches!(state.log[2].level, LogLevel::Agent));
        assert_eq!(state.log[2].source, "agent:coder-1");
        assert!(matches!(state.log[5].level, LogLevel::Hook));
        assert!(matches!(state.log[6].level, LogLevel::Error));
    }

    #[test]
    fn handles_empty_content() {
        let state = parse_progress("");
        assert!(matches!(state.status, OrchestrationStatus::Idle));
        assert_eq!(state.current_phase, 0);
        assert!(state.agents.is_empty());
        assert!(state.tasks.is_empty());
    }
}
