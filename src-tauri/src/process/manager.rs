use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::io::AsyncWriteExt;
use tokio::process::{Child, Command};
use tokio::sync::Mutex as AsyncMutex;
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::process::bridge;

/// Kills the child process when dropped.  Passed into the reader task so that
/// aborting the task also cleans up the spawned CLI process.
pub struct ChildGuard(pub Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.start_kill();
    }
}

/// Per-session state.
struct SessionHandle {
    /// The Claude CLI session ID returned in the stream-json `system` event.
    /// Shared with the reader task which updates it on each response.
    claude_session_id: Arc<AsyncMutex<Option<String>>>,
    /// Working directory for every spawned CLI process in this session.
    working_dir: Option<PathBuf>,
    /// Currently-running stdout reader task (if a message is being processed).
    active_reader: Option<JoinHandle<()>>,
    /// Currently-running stderr reader task.
    active_stderr_reader: Option<JoinHandle<()>>,
}

/// Manages Claude Code CLI interactions — one logical session at a time,
/// but each message spawns a short-lived `claude --print` process.
pub struct ProcessManager {
    sessions: HashMap<Uuid, SessionHandle>,
    cli_path: String,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            cli_path: "claude".to_string(),
        }
    }

    pub fn set_cli_path(&mut self, path: &str) {
        self.cli_path = path.to_string();
    }

    pub fn has_session(&self, session_id: &Uuid) -> bool {
        self.sessions.contains_key(session_id)
    }

    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Register a new session.  No process is spawned until `send()` is called.
    pub fn register_session(
        &mut self,
        session_id: Uuid,
        working_dir: Option<PathBuf>,
    ) -> Result<(), String> {
        if self.sessions.contains_key(&session_id) {
            return Err("Session already exists".to_string());
        }
        self.sessions.insert(
            session_id,
            SessionHandle {
                claude_session_id: Arc::new(AsyncMutex::new(None)),
                working_dir,
                active_reader: None,
                active_stderr_reader: None,
            },
        );
        Ok(())
    }

    /// Send a message for the given session.
    ///
    /// Spawns a new `claude --print --verbose --output-format stream-json`
    /// process, writes the message to stdin, closes stdin (triggering
    /// processing), and starts background reader tasks that emit Tauri events.
    ///
    /// For multi-turn conversations, `--resume <id>` is automatically added
    /// once the first response provides a Claude session ID.
    pub async fn send(
        &mut self,
        session_id: &Uuid,
        message: &str,
        app: tauri::AppHandle,
    ) -> Result<(), String> {
        // Clone cli_path before taking a mutable borrow on sessions
        let cli_path = self.cli_path.clone();

        let handle = self
            .sessions
            .get_mut(session_id)
            .ok_or("Session not found")?;

        // Abort any in-flight readers from a previous message
        if let Some(reader) = handle.active_reader.take() {
            reader.abort();
        }
        if let Some(reader) = handle.active_stderr_reader.take() {
            reader.abort();
        }

        // ── Build CLI command ────────────────────────────────────────────────

        let parts: Vec<&str> = cli_path.split_whitespace().collect();
        let (executable, extra_args) = parts
            .split_first()
            .ok_or_else(|| "CLI path is empty".to_string())?;

        let mut cmd = Command::new(executable);
        cmd.args(extra_args)
            .arg("--print")
            .arg("--verbose")
            .arg("--output-format")
            .arg("stream-json")
            .arg("--include-partial-messages")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        // Resume conversation if we have a session ID from a previous turn
        let claude_sid = handle.claude_session_id.lock().await.clone();
        if let Some(ref sid) = claude_sid {
            cmd.arg("--resume").arg(sid);
        }

        // macOS GUI apps don't inherit the user's shell PATH, so common
        // install locations won't be found.  Augment PATH.
        if !executable.contains('/') {
            let home = std::env::var("HOME").unwrap_or_default();
            let extra_paths = [
                format!("{}/.local/bin", home),
                format!("{}/.cargo/bin", home),
                "/usr/local/bin".to_string(),
                "/opt/homebrew/bin".to_string(),
            ];
            let current_path = std::env::var("PATH").unwrap_or_default();
            let combined = extra_paths
                .iter()
                .chain(std::iter::once(&current_path))
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(":");
            cmd.env("PATH", combined);
        }

        if let Some(ref dir) = handle.working_dir {
            cmd.current_dir(dir);
        }

        // ── Spawn and write message ──────────────────────────────────────────

        eprintln!("[helm:manager] spawning: {} {:?}", executable, cmd.as_std().get_args().collect::<Vec<_>>());

        let mut child = cmd
            .spawn()
            .map_err(|e| format!("Failed to spawn '{}': {}", executable, e))?;
        eprintln!("[helm:manager] spawned pid={:?}", child.id());

        // Write message to stdin, then close it so the CLI starts processing
        eprintln!("[helm:manager] writing to stdin: {:?}", &message[..message.len().min(100)]);
        {
            let mut stdin = child.stdin.take().ok_or("Failed to capture stdin")?;
            stdin
                .write_all(message.as_bytes())
                .await
                .map_err(|e| format!("stdin write failed: {}", e))?;
            stdin
                .write_all(b"\n")
                .await
                .map_err(|e| format!("stdin newline failed: {}", e))?;
            stdin
                .flush()
                .await
                .map_err(|e| format!("stdin flush failed: {}", e))?;
            // stdin is dropped here, closing the pipe and signaling EOF
        }

        let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
        let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;

        // ── Start reader tasks ───────────────────────────────────────────────

        let stderr_reader = bridge::spawn_stderr_reader(*session_id, stderr, app.clone());
        let session_id_arc = handle.claude_session_id.clone();
        let reader = bridge::spawn_json_reader(
            *session_id,
            stdout,
            ChildGuard(child),
            session_id_arc,
            app,
        );

        handle.active_reader = Some(reader);
        handle.active_stderr_reader = Some(stderr_reader);

        Ok(())
    }

    /// Kill/remove a session, aborting any in-flight readers.
    pub async fn kill(&mut self, session_id: &Uuid) -> Result<(), String> {
        if let Some(mut handle) = self.sessions.remove(session_id) {
            if let Some(reader) = handle.active_reader.take() {
                reader.abort();
            }
            if let Some(reader) = handle.active_stderr_reader.take() {
                reader.abort();
            }
            Ok(())
        } else {
            Err("No process for this session".to_string())
        }
    }

    /// Kill all running sessions (used on app shutdown).
    pub async fn kill_all(&mut self) {
        let ids: Vec<Uuid> = self.sessions.keys().copied().collect();
        for id in ids {
            let _ = self.kill(&id).await;
        }
    }

    /// Synchronous cleanup for use in the app exit handler where we cannot
    /// `.await`.  Aborts all reader tasks — the `ChildGuard` drop will
    /// kill any in-flight processes.
    pub fn kill_all_sync(&mut self) {
        for (_id, mut handle) in self.sessions.drain() {
            if let Some(reader) = handle.active_reader.take() {
                reader.abort();
            }
            if let Some(reader) = handle.active_stderr_reader.take() {
                reader.abort();
            }
        }
    }
}
