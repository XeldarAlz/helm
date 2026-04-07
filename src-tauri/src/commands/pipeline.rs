use std::path::Path;
use std::sync::Mutex;

use tauri::State;

use crate::models::pipeline::{
    AgentHealth, AssetCounts, FileNode, MailboxMessage, OrchLiveLog, OrchestrationState,
    OrchestrationStatus, PipelineState,
};
use crate::parser::progress::{parse_events_jsonl, parse_progress};
use crate::state::app_state::AppState;
use crate::watcher::docs::DocsWatcher;

#[tauri::command]
pub fn get_pipeline_state(state: State<AppState>) -> Result<PipelineState, String> {
    let project_dir = state.project_dir.lock().map_err(|e| e.to_string())?;

    let base = match project_dir.as_ref() {
        Some(dir) => dir.join("docs"),
        None => {
            return Ok(PipelineState {
                gdd_exists: false,
                tdd_exists: false,
                workflow_exists: false,
                project_claude_md_exists: false,
                progress_exists: false,
                catch_up_exists: false,
                current_phase: "none".to_string(),
            });
        }
    };

    let root = project_dir.as_ref().unwrap(); // safe — None handled above
    let gdd = base.join("GDD.md").exists();
    let tdd = base.join("TDD.md").exists();
    let workflow = base.join("WORKFLOW.md").exists();
    let project_claude_md = root.join(".claude").join("CLAUDE.md").exists();
    let progress = base.join("PROGRESS.md").exists() || root.join("PROGRESS.md").exists();
    let catch_up = base.join("CATCH_UP.md").exists();

    let phase = if progress {
        "building"
    } else if project_claude_md && workflow {
        "initialized"
    } else if workflow {
        "planning"
    } else if tdd {
        "architecture"
    } else if gdd {
        "idea"
    } else {
        "none"
    };

    Ok(PipelineState {
        gdd_exists: gdd,
        tdd_exists: tdd,
        workflow_exists: workflow,
        project_claude_md_exists: project_claude_md,
        progress_exists: progress,
        catch_up_exists: catch_up,
        current_phase: phase.to_string(),
    })
}

#[tauri::command]
pub fn read_document(
    state: State<AppState>,
    name: String,
) -> Result<String, String> {
    let project_dir = state.project_dir.lock().map_err(|e| e.to_string())?;

    let base = project_dir
        .as_ref()
        .ok_or("No project directory set")?
        .join("docs");

    let filename = match name.as_str() {
        "GDD" => "GDD.md",
        "TDD" => "TDD.md",
        "WORKFLOW" => "WORKFLOW.md",
        "PROGRESS" => "PROGRESS.md",
        "ACTIVITY_LOG" => "ACTIVITY_LOG.md",
        "CATCH_UP" => "CATCH_UP.md",
        _ => return Err(format!("Unknown document: {}", name)),
    };

    let path = base.join(filename);
    std::fs::read_to_string(&path).map_err(|e| format!("Failed to read {}: {}", filename, e))
}

/// Recursively count files matching a given extension under a directory.
fn count_files(dir: &Path, extension: &str) -> u32 {
    let mut count = 0u32;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                count += count_files(&path, extension);
            } else if path.extension().is_some_and(|ext| ext == extension) {
                count += 1;
            }
        }
    }
    count
}

#[tauri::command]
pub fn get_asset_counts(state: State<AppState>) -> Result<AssetCounts, String> {
    let project_dir = state.project_dir.lock().map_err(|e| e.to_string())?;

    let base = match project_dir.as_ref() {
        Some(dir) => dir.join("Assets"),
        None => {
            return Ok(AssetCounts {
                scripts: 0,
                tests: 0,
                prefabs: 0,
                configs: 0,
            });
        }
    };

    if !base.exists() {
        return Ok(AssetCounts {
            scripts: 0,
            tests: 0,
            prefabs: 0,
            configs: 0,
        });
    }

    // Count .cs files, separating tests from scripts by checking if
    // any ancestor directory is named "Tests" or "Editor".
    let mut scripts = 0u32;
    let mut tests = 0u32;
    count_cs_files(&base, &mut scripts, &mut tests);

    Ok(AssetCounts {
        scripts,
        tests,
        prefabs: count_files(&base, "prefab"),
        configs: count_files(&base, "asset"),
    })
}

/// Recursively count .cs files, classifying them as tests or scripts based
/// on whether they live under a "Tests", "Test", or "Editor" directory.
fn count_cs_files(dir: &Path, scripts: &mut u32, tests: &mut u32) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            count_cs_files(&path, scripts, tests);
        } else if path.extension().is_some_and(|ext| ext == "cs") {
            if is_test_path(&path) {
                *tests += 1;
            } else {
                *scripts += 1;
            }
        }
    }
}

/// Check if a path is under a test-related directory.
fn is_test_path(path: &Path) -> bool {
    path.components().any(|c| {
        let s = c.as_os_str().to_string_lossy();
        s == "Tests" || s == "Test" || s == "Editor"
    })
}

#[tauri::command]
pub async fn check_cli(state: State<'_, AppState>) -> Result<bool, String> {
    let cli_path = {
        let settings = state.settings.lock().map_err(|e| e.to_string())?;
        settings.claude_cli_path.clone()
    };

    let parts: Vec<&str> = cli_path.split_whitespace().collect();
    let (executable, extra_args) = parts
        .split_first()
        .ok_or_else(|| "CLI path is empty".to_string())?;

    match tokio::process::Command::new(executable)
        .args(extra_args)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .await
    {
        Ok(status) => Ok(status.success()),
        Err(_) => Ok(false),
    }
}

// ── File Watcher Commands ───────────────────────────────────────────────────

pub struct WatcherState(pub Mutex<DocsWatcher>);

#[tauri::command]
pub fn start_watching(
    app_state: State<AppState>,
    watcher_state: State<WatcherState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let project_dir = app_state.project_dir.lock().map_err(|e| e.to_string())?;
    let dir = project_dir
        .as_ref()
        .ok_or("No project directory set")?;

    let mut watcher = watcher_state.0.lock().map_err(|e| e.to_string())?;
    watcher.start(dir, app)
}

#[tauri::command]
pub fn stop_watching(watcher_state: State<WatcherState>) -> Result<(), String> {
    let mut watcher = watcher_state.0.lock().map_err(|e| e.to_string())?;
    watcher.stop();
    Ok(())
}

// ── File Tree Commands (Code Browser) ───────────────────────────────────────

/// Recursively build a file tree, skipping hidden dirs, node_modules, etc.
fn build_file_tree(dir: &Path, base: &Path, max_depth: u32, depth: u32) -> Vec<FileNode> {
    if depth > max_depth {
        return Vec::new();
    }

    let mut nodes: Vec<FileNode> = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return nodes;
    };

    let mut entries: Vec<_> = entries.flatten().collect();
    entries.sort_by_key(|e| {
        let is_dir = e.path().is_dir();
        // Directories first, then files, alphabetically
        (!is_dir, e.file_name().to_string_lossy().to_lowercase())
    });

    for entry in entries {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip hidden files/dirs and common irrelevant directories
        if name.starts_with('.')
            || name == "node_modules"
            || name == "target"
            || name == "dist"
            || name == "Library"
            || name == "Temp"
            || name == "Logs"
            || name == "obj"
            || name == "bin"
        {
            continue;
        }

        let relative_path = path
            .strip_prefix(base)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        let is_dir = path.is_dir();
        let children = if is_dir {
            build_file_tree(&path, base, max_depth, depth + 1)
        } else {
            Vec::new()
        };

        nodes.push(FileNode {
            name,
            path: relative_path,
            is_dir,
            children,
            extension: if is_dir {
                None
            } else {
                path.extension().map(|e| e.to_string_lossy().to_string())
            },
            size: if is_dir {
                None
            } else {
                std::fs::metadata(&path).ok().map(|m| m.len())
            },
        });
    }

    nodes
}

#[tauri::command]
pub fn get_file_tree(
    state: State<AppState>,
    root: Option<String>,
) -> Result<Vec<FileNode>, String> {
    let project_dir = state.project_dir.lock().map_err(|e| e.to_string())?;
    let base = project_dir
        .as_ref()
        .ok_or("No project directory set")?;

    let scan_dir = match root {
        Some(ref sub) => base.join(sub),
        None => base.clone(),
    };

    if !scan_dir.exists() {
        return Ok(Vec::new());
    }

    Ok(build_file_tree(&scan_dir, base, 8, 0))
}

#[tauri::command]
pub fn read_project_file(
    state: State<AppState>,
    path: String,
) -> Result<String, String> {
    let project_dir = state.project_dir.lock().map_err(|e| e.to_string())?;
    let base = project_dir
        .as_ref()
        .ok_or("No project directory set")?;

    let file_path = base.join(&path);

    // Security: ensure path doesn't escape project directory
    let canonical_base = base
        .canonicalize()
        .map_err(|e| format!("Failed to resolve project dir: {}", e))?;
    let canonical_file = file_path
        .canonicalize()
        .map_err(|e| format!("Failed to resolve file path: {}", e))?;

    if !canonical_file.starts_with(&canonical_base) {
        return Err("Path traversal not allowed".to_string());
    }

    std::fs::read_to_string(&canonical_file)
        .map_err(|e| format!("Failed to read {}: {}", path, e))
}

// ── Phase 6: Orchestration Commands ─────────────────────────────────────────

#[tauri::command]
pub fn get_orchestration_state(
    state: State<AppState>,
    live_log: State<OrchLiveLog>,
) -> Result<OrchestrationState, String> {
    let project_dir = state.project_dir.lock().map_err(|e| e.to_string())?;

    let root = match project_dir.as_ref() {
        Some(dir) => dir.clone(),
        None => return Ok(OrchestrationState::default()),
    };

    // Check docs/PROGRESS.md first (canonical location), then fall back to
    // ./PROGRESS.md at the project root (agents sometimes write there if
    // they lack write access to the docs/ directory).
    let candidates = [
        root.join("docs").join("PROGRESS.md"),
        root.join("PROGRESS.md"),
    ];

    let progress_path = candidates.iter().find(|p| p.exists());

    let mut state = if let Some(path) = progress_path {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read PROGRESS.md: {}", e))?;
        parse_progress(&content)
    } else {
        OrchestrationState::default()
    };

    // If PROGRESS.md yielded no data, fall back to EVENTS.jsonl which is
    // the authoritative event journal and is typically written before
    // PROGRESS.md is first created.
    if state.log.is_empty() && state.agents.is_empty() {
        let events_candidates = [
            root.join("docs").join("EVENTS.jsonl"),
            root.join("EVENTS.jsonl"),
        ];
        if let Some(events_path) = events_candidates.iter().find(|p| p.exists()) {
            if let Ok(content) = std::fs::read_to_string(events_path) {
                let events_state = parse_events_jsonl(&content);
                if !events_state.log.is_empty() {
                    state = events_state;
                }
            }
        }
    }

    // Enrich agents with heartbeat and mailbox data
    let claude_dir = root.join(".claude");
    for agent in &mut state.agents {
        // Read heartbeat
        let heartbeat_path = claude_dir.join("heartbeat").join(format!("{}.json", agent.id));
        if let Ok(hb_content) = std::fs::read_to_string(&heartbeat_path) {
            if let Ok(hb) = serde_json::from_str::<serde_json::Value>(&hb_content) {
                if let Some(ts) = hb.get("ts").and_then(|v| v.as_str()) {
                    // Parse timestamp and check staleness
                    if let Ok(hb_time) = chrono::DateTime::parse_from_rfc3339(ts) {
                        let age = chrono::Utc::now().signed_duration_since(hb_time);
                        agent.health = if age.num_minutes() > 20 {
                            AgentHealth::Dead
                        } else if age.num_minutes() > 10 {
                            AgentHealth::Stale
                        } else {
                            AgentHealth::Alive
                        };
                    }
                }
            }
        }

        // Read last mailbox message
        let mailbox_path = claude_dir.join("mailbox").join(format!("{}.jsonl", agent.id));
        if let Ok(mb_content) = std::fs::read_to_string(&mailbox_path) {
            if let Some(last_line) = mb_content.lines().filter(|l| !l.trim().is_empty()).last() {
                if let Ok(msg) = serde_json::from_str::<MailboxMessage>(last_line) {
                    agent.last_mailbox = Some(msg);
                }
            }
        }
    }

    // Stop protection: check if orchestration-active.json exists
    let orch_active_path = claude_dir.join("orchestration-active.json");
    state.stop_protected = orch_active_path.exists();

    // Pre-compact state: extract "## Saved:" timestamp
    let pre_compact_path = claude_dir.join("pre-compact-state.md");
    if pre_compact_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&pre_compact_path) {
            for line in content.lines() {
                if let Some(ts) = line.trim().strip_prefix("## Saved:") {
                    let ts = ts.trim();
                    if !ts.is_empty() {
                        state.last_compact_save = Some(ts.to_string());
                    }
                    break;
                }
            }
        }
    }

    // Orchestrator checkpoint: extract "## Updated:" timestamp
    let orch_state_path = claude_dir.join("orchestrator-state.md");
    if orch_state_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&orch_state_path) {
            for line in content.lines() {
                if let Some(ts) = line.trim().strip_prefix("## Updated:") {
                    let ts = ts.trim();
                    if !ts.is_empty() {
                        state.orchestrator_checkpoint = Some(ts.to_string());
                    }
                    break;
                }
            }
        }
    }

    // Merge live events detected from session text output.  These fill
    // the gap when the orchestrator agent hasn't written to PROGRESS.md
    // or EVENTS.jsonl (which is surprisingly common).
    let live = live_log.snapshot();
    if !live.log.is_empty() {
        // Build a set of existing messages to avoid duplicates
        let existing: std::collections::HashSet<String> = state
            .log
            .iter()
            .map(|e| format!("{}:{}", e.timestamp, e.message))
            .collect();

        for entry in &live.log {
            let key = format!("{}:{}", entry.timestamp, entry.message);
            if !existing.contains(&key) {
                state.log.push(entry.clone());
            }
        }

        // Promote status from idle to running if live events exist
        if matches!(state.status, OrchestrationStatus::Idle) {
            if let Some(ref live_status) = live.status {
                state.status = live_status.clone();
            }
        }

        // Use live phase if file-based state has no phase info
        if state.current_phase == 0 && live.current_phase > 0 {
            state.current_phase = live.current_phase;
        }
    }

    // Sort log by timestamp for consistent display
    state
        .log
        .sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    Ok(state)
}

#[tauri::command]
pub async fn send_orchestration_command(
    state: State<'_, AppState>,
    process_mgr: State<'_, crate::commands::session::ProcessMgr>,
    app: tauri::AppHandle,
    command: String,
) -> Result<(), String> {
    // Find active orchestration session
    let session_id = {
        let sessions = state.sessions.lock().map_err(|e| e.to_string())?;
        sessions
            .iter()
            .find(|(_, s)| {
                matches!(s.phase, crate::models::session::PipelinePhase::Orchestrate)
                    && matches!(s.status, crate::models::session::SessionStatus::Active)
            })
            .map(|(id, _)| *id)
            .ok_or("No active orchestration session")?
    };

    let cmd = match command.as_str() {
        "pause" => "/stop",
        "resume" => "/continue",
        "stop" => "/stop",
        "clean-slop" => "/clean-slop",
        _ => return Err(format!("Unknown orchestration command: {}", command)),
    };

    let mut mgr = process_mgr.lock().await;
    mgr.send(&session_id, cmd, app).await
}
