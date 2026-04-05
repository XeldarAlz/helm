use std::path::{Path, PathBuf};

use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[derive(Clone, Serialize)]
pub struct DocChangedPayload {
    pub name: String,
    pub path: String,
}

#[derive(Clone, Serialize)]
pub struct CodeChangedPayload {
    pub path: String,
    pub kind: String,
}

pub struct DocsWatcher {
    watcher: Option<RecommendedWatcher>,
    watched_paths: Vec<PathBuf>,
}

impl DocsWatcher {
    pub fn new() -> Self {
        Self {
            watcher: None,
            watched_paths: Vec::new(),
        }
    }

    /// Start watching the docs/ directory and optionally Assets/Scripts/.
    pub fn start(&mut self, project_dir: &Path, app: AppHandle) -> Result<(), String> {
        self.stop();

        let app_clone = app.clone();
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    if !matches!(
                        event.kind,
                        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                    ) {
                        return;
                    }

                    for path in &event.paths {
                        let filename = path
                            .file_name()
                            .map(|f| f.to_string_lossy().to_string())
                            .unwrap_or_default();

                        let event_name = match filename.as_str() {
                            "PROGRESS.md" => "progress-updated",
                            "ACTIVITY_LOG.md" => "activity-logged",
                            "GDD.md" | "TDD.md" | "WORKFLOW.md" | "CLAUDE.md"
                            | "CATCH_UP.md" => "document-updated",
                            _ if path
                                .extension()
                                .is_some_and(|ext| ext == "cs" || ext == "rs") =>
                            {
                                let _ = app_clone.emit(
                                    "code-changed",
                                    CodeChangedPayload {
                                        path: path.to_string_lossy().to_string(),
                                        kind: format!("{:?}", event.kind),
                                    },
                                );
                                continue;
                            }
                            _ => continue,
                        };

                        let _ = app_clone.emit(
                            event_name,
                            DocChangedPayload {
                                name: filename,
                                path: path.to_string_lossy().to_string(),
                            },
                        );
                    }
                }
                Err(e) => {
                    eprintln!("File watch error: {}", e);
                }
            }
        })
        .map_err(|e| format!("Failed to create watcher: {}", e))?;

        // Watch docs/ directory
        let docs_dir = project_dir.join("docs");
        if docs_dir.exists() {
            watcher
                .watch(&docs_dir, RecursiveMode::NonRecursive)
                .map_err(|e| format!("Failed to watch docs/: {}", e))?;
            self.watched_paths.push(docs_dir);
        }

        // Also watch the project root (non-recursive) so we catch
        // PROGRESS.md written at the root level by agents that lack
        // write access to docs/.
        watcher
            .watch(project_dir, RecursiveMode::NonRecursive)
            .map_err(|e| format!("Failed to watch project root: {}", e))?;
        self.watched_paths.push(project_dir.to_path_buf());

        // Watch .claude/ directory for project CLAUDE.md changes
        let claude_dir = project_dir.join(".claude");
        if claude_dir.exists() {
            watcher
                .watch(&claude_dir, RecursiveMode::NonRecursive)
                .map_err(|e| format!("Failed to watch .claude/: {}", e))?;
            self.watched_paths.push(claude_dir);
        }

        // Watch Assets/Scripts/ if it exists (for code changes)
        let scripts_dir = project_dir.join("Assets").join("Scripts");
        if scripts_dir.exists() {
            watcher
                .watch(&scripts_dir, RecursiveMode::Recursive)
                .map_err(|e| format!("Failed to watch Assets/Scripts/: {}", e))?;
            self.watched_paths.push(scripts_dir);
        }

        self.watcher = Some(watcher);
        Ok(())
    }

    pub fn stop(&mut self) {
        self.watcher = None;
        self.watched_paths.clear();
    }

    pub fn is_watching(&self) -> bool {
        self.watcher.is_some()
    }
}
