use std::collections::HashMap;
use std::process::Command;

use tauri::State;

use crate::models::git::{GitBranch, GitCommit, GitDiffFile, GitStatus, GitStatusEntry};
use crate::state::app_state::AppState;

/// Run a git command in the project directory, returning stdout as String.
fn git_cmd(project_dir: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(project_dir)
        .output()
        .map_err(|e| format!("Failed to run git: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git error: {}", stderr.trim()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn resolve_project_dir(state: &AppState) -> Result<String, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    if settings.project_dir.is_empty() {
        let project_dir = state.project_dir.lock().map_err(|e| e.to_string())?;
        project_dir
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .ok_or_else(|| "No project directory set".to_string())
    } else {
        Ok(settings.project_dir.clone())
    }
}

#[tauri::command]
pub fn get_git_log(
    state: State<AppState>,
    count: Option<u32>,
) -> Result<Vec<GitCommit>, String> {
    let dir = resolve_project_dir(&state)?;
    let limit = count.unwrap_or(50).to_string();

    let output = git_cmd(
        &dir,
        &[
            "log",
            &format!("-{}", limit),
            "--format=%H%n%h%n%an%n%aI%n%s",
            "--shortstat",
        ],
    )?;

    let mut commits = Vec::new();
    let mut lines = output.lines().peekable();

    while lines.peek().is_some() {
        // Skip empty lines between entries
        while lines.peek().is_some_and(|l| l.is_empty()) {
            lines.next();
        }

        let Some(hash) = lines.next() else { break };
        if hash.is_empty() {
            continue;
        }
        let short_hash = lines.next().unwrap_or_default().to_string();
        let author = lines.next().unwrap_or_default().to_string();
        let date = lines.next().unwrap_or_default().to_string();
        let message = lines.next().unwrap_or_default().to_string();

        // --shortstat line (may be empty for commits with no file changes)
        let mut files_changed = 0u32;
        let mut insertions = 0u32;
        let mut deletions = 0u32;

        if let Some(stat_line) = lines.peek() {
            if stat_line.contains("file") || stat_line.contains("insertion") || stat_line.contains("deletion") {
                let stat = lines.next().unwrap_or_default();
                // Parse "3 files changed, 10 insertions(+), 2 deletions(-)"
                for part in stat.split(',') {
                    let part = part.trim();
                    if part.contains("file") {
                        files_changed = part.split_whitespace().next().and_then(|n| n.parse().ok()).unwrap_or(0);
                    } else if part.contains("insertion") {
                        insertions = part.split_whitespace().next().and_then(|n| n.parse().ok()).unwrap_or(0);
                    } else if part.contains("deletion") {
                        deletions = part.split_whitespace().next().and_then(|n| n.parse().ok()).unwrap_or(0);
                    }
                }
            }
        }

        commits.push(GitCommit {
            hash: hash.to_string(),
            short_hash,
            author,
            date,
            message,
            files_changed,
            insertions,
            deletions,
        });
    }

    Ok(commits)
}

#[tauri::command]
pub fn get_git_branches(state: State<AppState>) -> Result<Vec<GitBranch>, String> {
    let dir = resolve_project_dir(&state)?;

    let output = git_cmd(
        &dir,
        &["branch", "--format=%(HEAD) %(refname:short) %(objectname:short) %(subject)"],
    )?;

    let branches: Vec<GitBranch> = output
        .lines()
        .filter(|l| !l.is_empty())
        .map(|line| {
            let is_current = line.starts_with('*');
            let rest = line.trim_start_matches(['*', ' ']);
            let mut parts = rest.splitn(3, ' ');
            let name = parts.next().unwrap_or_default().to_string();
            let _hash = parts.next().unwrap_or_default();
            let last_commit = parts.next().unwrap_or_default().to_string();

            GitBranch {
                name,
                is_current,
                last_commit,
            }
        })
        .collect();

    Ok(branches)
}

#[tauri::command]
pub fn get_git_status(state: State<AppState>) -> Result<GitStatus, String> {
    let dir = resolve_project_dir(&state)?;

    // Get current branch
    let branch = git_cmd(&dir, &["branch", "--show-current"])?
        .trim()
        .to_string();

    // Get ahead/behind
    let mut ahead = 0u32;
    let mut behind = 0u32;
    if let Ok(ab) = git_cmd(&dir, &["rev-list", "--left-right", "--count", "HEAD...@{upstream}"]) {
        let parts: Vec<&str> = ab.trim().split('\t').collect();
        if parts.len() == 2 {
            ahead = parts[0].parse().unwrap_or(0);
            behind = parts[1].parse().unwrap_or(0);
        }
    }

    // Porcelain status
    let status_output = git_cmd(&dir, &["status", "--porcelain=v1"])?;

    let mut staged = Vec::new();
    let mut unstaged = Vec::new();
    let mut untracked = Vec::new();

    for line in status_output.lines() {
        if line.len() < 4 {
            continue;
        }
        let index_status = line.chars().nth(0).unwrap_or(' ');
        let work_status = line.chars().nth(1).unwrap_or(' ');
        let path = line[3..].to_string();

        if index_status == '?' {
            untracked.push(path);
        } else {
            if index_status != ' ' {
                staged.push(GitStatusEntry {
                    path: path.clone(),
                    status: index_status.to_string(),
                });
            }
            if work_status != ' ' {
                unstaged.push(GitStatusEntry {
                    path,
                    status: work_status.to_string(),
                });
            }
        }
    }

    Ok(GitStatus {
        branch,
        ahead,
        behind,
        staged,
        unstaged,
        untracked,
    })
}

#[tauri::command]
pub fn get_git_diff(
    state: State<AppState>,
    commit_hash: String,
) -> Result<Vec<GitDiffFile>, String> {
    let dir = resolve_project_dir(&state)?;

    let output = git_cmd(
        &dir,
        &["diff-tree", "--no-commit-id", "-r", "--numstat", &commit_hash],
    )?;

    let files: Vec<GitDiffFile> = output
        .lines()
        .filter(|l| !l.is_empty())
        .map(|line| {
            let parts: Vec<&str> = line.split('\t').collect();
            let insertions = parts.first().and_then(|s| s.parse().ok()).unwrap_or(0);
            let deletions = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
            let path = parts.get(2).unwrap_or(&"").to_string();

            let status = if insertions > 0 && deletions > 0 {
                "M"
            } else if insertions > 0 {
                "A"
            } else {
                "D"
            };

            GitDiffFile {
                path,
                status: status.to_string(),
                insertions,
                deletions,
            }
        })
        .collect();

    Ok(files)
}

#[tauri::command]
pub fn get_commit_trailers(
    state: State<AppState>,
    commit_hash: String,
) -> Result<HashMap<String, String>, String> {
    let dir = resolve_project_dir(&state)?;
    let output = git_cmd(&dir, &["log", "--format=%B", "-1", &commit_hash])?;

    let mut trailers = HashMap::new();
    let known = [
        "Constraint",
        "Rejected",
        "Confidence",
        "Scope-risk",
        "Not-tested",
    ];

    for line in output.lines().rev() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            break;
        }
        if let Some((key, value)) = trimmed.split_once(':') {
            let key = key.trim();
            if known.iter().any(|k| k.eq_ignore_ascii_case(key)) {
                trailers.insert(key.to_string(), value.trim().to_string());
            }
        }
    }

    Ok(trailers)
}
