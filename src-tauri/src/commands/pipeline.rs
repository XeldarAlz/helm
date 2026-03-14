use std::path::Path;
use std::sync::Mutex;

use tauri::State;

use crate::models::pipeline::{AssetCounts, FileNode, OrchestrationState, PipelineState};
use crate::parser::progress::parse_progress;
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
                progress_exists: false,
                current_phase: "none".to_string(),
            });
        }
    };

    let gdd = base.join("GDD.md").exists();
    let tdd = base.join("TDD.md").exists();
    let workflow = base.join("WORKFLOW.md").exists();
    let progress = base.join("PROGRESS.md").exists();

    let phase = if progress {
        "building"
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
        progress_exists: progress,
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

    Ok(AssetCounts {
        scripts: count_files(&base.join("Scripts"), "cs"),
        tests: count_files(&base.join("Tests"), "cs"),
        prefabs: count_files(&base.join("Prefabs"), "prefab"),
        configs: count_files(&base.join("ScriptableObjects"), "asset")
            + count_files(&base.join("Resources"), "asset"),
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
) -> Result<OrchestrationState, String> {
    let project_dir = state.project_dir.lock().map_err(|e| e.to_string())?;

    let base = match project_dir.as_ref() {
        Some(dir) => dir.join("docs"),
        None => return Ok(OrchestrationState::default()),
    };

    let progress_path = base.join("PROGRESS.md");
    if !progress_path.exists() {
        return Ok(OrchestrationState::default());
    }

    let content = std::fs::read_to_string(&progress_path)
        .map_err(|e| format!("Failed to read PROGRESS.md: {}", e))?;

    Ok(parse_progress(&content))
}

#[tauri::command]
pub async fn send_orchestration_command(
    state: State<'_, AppState>,
    process_mgr: State<'_, crate::commands::session::ProcessMgr>,
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
        "resume" => "/resume",
        "stop" => "/stop",
        _ => return Err(format!("Unknown orchestration command: {}", command)),
    };

    let mut mgr = process_mgr.lock().await;
    mgr.send(&session_id, cmd).await
}
