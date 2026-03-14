use std::collections::HashMap;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, ChildStdin, Command};
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::process::bridge;

/// Handle to a running Claude Code child process.
struct ProcessHandle {
    child: Child,
    stdin: ChildStdin,
    stdout_reader: JoinHandle<()>,
    stderr_reader: JoinHandle<()>,
    pub started_at: DateTime<Utc>,
}

/// Manages Claude Code CLI child processes — one per session.
pub struct ProcessManager {
    processes: HashMap<Uuid, ProcessHandle>,
    cli_path: String,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            processes: HashMap::new(),
            cli_path: "claude".to_string(),
        }
    }

    pub fn set_cli_path(&mut self, path: &str) {
        self.cli_path = path.to_string();
    }

    pub fn has_session(&self, session_id: &Uuid) -> bool {
        self.processes.contains_key(session_id)
    }

    pub fn session_count(&self) -> usize {
        self.processes.len()
    }

    /// Spawn a new Claude Code CLI process for the given session.
    /// Pipes stdin/stdout/stderr and starts background reader tasks
    /// that emit Tauri events to the frontend.
    pub async fn spawn(
        &mut self,
        session_id: Uuid,
        working_dir: Option<PathBuf>,
        app: tauri::AppHandle,
    ) -> Result<(), String> {
        if self.processes.contains_key(&session_id) {
            return Err("Session already has a running process".to_string());
        }

        let mut cmd = Command::new(&self.cli_path);
        cmd.stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn '{}': {}", self.cli_path, e))?;

        let stdin = child
            .stdin
            .take()
            .ok_or("Failed to capture stdin")?;
        let stdout = child
            .stdout
            .take()
            .ok_or("Failed to capture stdout")?;
        let stderr = child
            .stderr
            .take()
            .ok_or("Failed to capture stderr")?;

        let stdout_reader = bridge::spawn_stdout_reader(session_id, stdout, app.clone());
        let stderr_reader = bridge::spawn_stderr_reader(session_id, stderr, app);

        self.processes.insert(
            session_id,
            ProcessHandle {
                child,
                stdin,
                stdout_reader,
                stderr_reader,
                started_at: Utc::now(),
            },
        );

        Ok(())
    }

    /// Write a message to the session's claude process stdin.
    pub async fn send(&mut self, session_id: &Uuid, message: &str) -> Result<(), String> {
        let handle = self
            .processes
            .get_mut(session_id)
            .ok_or("No process for this session")?;

        handle
            .stdin
            .write_all(message.as_bytes())
            .await
            .map_err(|e| format!("stdin write failed: {}", e))?;
        handle
            .stdin
            .write_all(b"\n")
            .await
            .map_err(|e| format!("stdin newline failed: {}", e))?;
        handle
            .stdin
            .flush()
            .await
            .map_err(|e| format!("stdin flush failed: {}", e))?;

        Ok(())
    }

    /// Kill the session's claude process and clean up reader tasks.
    pub async fn kill(&mut self, session_id: &Uuid) -> Result<(), String> {
        if let Some(mut handle) = self.processes.remove(session_id) {
            handle.stdout_reader.abort();
            handle.stderr_reader.abort();
            let _ = handle.child.kill().await;
            Ok(())
        } else {
            Err("No process for this session".to_string())
        }
    }

    /// Kill all running processes (used on app shutdown).
    pub async fn kill_all(&mut self) {
        let ids: Vec<Uuid> = self.processes.keys().copied().collect();
        for id in ids {
            let _ = self.kill(&id).await;
        }
    }
}
