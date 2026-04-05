use std::path::Path;

use include_dir::{include_dir, Dir};
use tauri::State;

use crate::state::app_state::AppState;

/// The entire `.claude/` directory, embedded at compile time.
static CLAUDE_CONFIG: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../.claude");

/// Check whether `.claude/CLAUDE.md` exists in the project directory.
#[tauri::command]
pub fn check_claude_config(state: State<AppState>) -> Result<bool, String> {
    let project_dir = state.project_dir.lock().map_err(|e| e.to_string())?;
    let dir = match project_dir.as_ref() {
        Some(d) => d,
        None => return Ok(false),
    };
    Ok(dir.join(".claude").join("CLAUDE.md").exists())
}

/// Copy the embedded `.claude/` tree into `{projectDir}/.claude/`.
///
/// Overwrites existing files but does not delete extras the user may have
/// added.  Returns the number of files written.
#[tauri::command]
pub fn inject_claude_config(state: State<AppState>) -> Result<u32, String> {
    let project_dir = state.project_dir.lock().map_err(|e| e.to_string())?;
    let dir = project_dir
        .as_ref()
        .ok_or("No project directory set")?;

    let target = dir.join(".claude");
    let count = extract_recursive(&CLAUDE_CONFIG, &target)?;

    // Make hook scripts executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let hooks_dir = target.join("hooks");
        if hooks_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&hooks_dir) {
                for entry in entries.flatten() {
                    if entry.path().extension().is_some_and(|e| e == "sh") {
                        if let Ok(meta) = std::fs::metadata(entry.path()) {
                            let mut perms = meta.permissions();
                            perms.set_mode(0o755);
                            let _ = std::fs::set_permissions(entry.path(), perms);
                        }
                    }
                }
            }
        }
    }

    eprintln!(
        "[helm:inject] wrote {} files to {:?}",
        count,
        target
    );

    Ok(count)
}

/// Recursively extract an embedded directory to disk, skipping `.DS_Store`.
fn extract_recursive(dir: &Dir<'_>, target: &Path) -> Result<u32, String> {
    std::fs::create_dir_all(target)
        .map_err(|e| format!("Failed to create {}: {}", target.display(), e))?;

    let mut count = 0u32;

    for file in dir.files() {
        let name = file.path().to_string_lossy();
        if name.contains(".DS_Store") {
            continue;
        }

        let dest = target.join(file.path());
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create dir {}: {}", parent.display(), e))?;
        }
        std::fs::write(&dest, file.contents())
            .map_err(|e| format!("Failed to write {}: {}", dest.display(), e))?;
        count += 1;
    }

    for subdir in dir.dirs() {
        count += extract_recursive(subdir, target)?;
    }

    Ok(count)
}
