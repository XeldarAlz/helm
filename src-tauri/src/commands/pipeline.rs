use tauri::State;

use crate::models::pipeline::PipelineState;
use crate::state::app_state::AppState;

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
        _ => return Err(format!("Unknown document: {}", name)),
    };

    let path = base.join(filename);
    std::fs::read_to_string(&path).map_err(|e| format!("Failed to read {}: {}", filename, e))
}
