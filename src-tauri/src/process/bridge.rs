use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{ChildStderr, ChildStdout};
use tokio::sync::Mutex as AsyncMutex;
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::process::manager::ChildGuard;
use crate::process::parser::{parse_stream_json, strip_ansi_codes, ClaudeEvent, OutputParser, StreamEvent};

// ── Event payloads emitted to the frontend ──────────────────────────────────

#[derive(Clone, Serialize)]
pub struct SessionOutputPayload {
    pub session_id: Uuid,
    pub text: String,
}

#[derive(Clone, Serialize)]
pub struct SessionQuestionPayload {
    pub session_id: Uuid,
    pub text: String,
    pub suggested_answers: Vec<String>,
}

#[derive(Clone, Serialize)]
pub struct SessionStreamingPayload {
    pub session_id: Uuid,
    pub streaming: bool,
}

#[derive(Clone, Serialize)]
pub struct SessionErrorPayload {
    pub session_id: Uuid,
    pub error: String,
}

#[derive(Clone, Serialize)]
pub struct ToolUsePayload {
    pub session_id: Uuid,
    pub tool: String,
    pub input_summary: String,
}

#[derive(Clone, Serialize)]
pub struct PermissionRequestPayload {
    pub session_id: Uuid,
    pub tool: String,
    pub description: String,
}

/// How long to wait for the next line before declaring a response complete.
/// 3 seconds covers most tool-use pauses (file reads, bash execution) while
/// still turning off the typing indicator promptly after a real response ends.
const RESPONSE_TIMEOUT: Duration = Duration::from_millis(3000);

/// Spawn a tokio task that reads the child process stdout line-by-line,
/// strips ANSI escape codes, parses it with OutputParser, and emits Tauri
/// events to the frontend.
pub fn spawn_stdout_reader(
    session_id: Uuid,
    stdout: ChildStdout,
    app: tauri::AppHandle,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        let mut parser = OutputParser::new();
        let mut is_streaming = false;

        loop {
            match tokio::time::timeout(RESPONSE_TIMEOUT, lines.next_line()).await {
                // Got a line of output
                Ok(Ok(Some(raw_line))) => {
                    // Strip ANSI escape codes before parsing
                    let line = strip_ansi_codes(&raw_line);

                    if !is_streaming {
                        is_streaming = true;
                        let _ = app.emit(
                            "session:streaming",
                            SessionStreamingPayload {
                                session_id,
                                streaming: true,
                            },
                        );
                    }

                    let events = parser.parse_line(&line);
                    for event in events {
                        emit_claude_event(&app, session_id, &event);
                    }
                }

                // EOF — child process exited
                Ok(Ok(None)) => {
                    // Flush any pending parser state
                    for event in parser.flush() {
                        emit_claude_event(&app, session_id, &event);
                    }

                    if is_streaming {
                        let _ = app.emit(
                            "session:streaming",
                            SessionStreamingPayload {
                                session_id,
                                streaming: false,
                            },
                        );
                    }

                    let _ = app.emit(
                        "session:complete",
                        SessionOutputPayload {
                            session_id,
                            text: String::new(),
                        },
                    );
                    break;
                }

                // IO error
                Ok(Err(e)) => {
                    let _ = app.emit(
                        "session:error",
                        SessionErrorPayload {
                            session_id,
                            error: format!("stdout read error: {}", e),
                        },
                    );
                    if is_streaming {
                        let _ = app.emit(
                            "session:streaming",
                            SessionStreamingPayload {
                                session_id,
                                streaming: false,
                            },
                        );
                    }
                    break;
                }

                // Timeout — response likely complete, keep listening
                Err(_) => {
                    for event in parser.flush() {
                        emit_claude_event(&app, session_id, &event);
                    }

                    if is_streaming {
                        is_streaming = false;
                        let _ = app.emit(
                            "session:streaming",
                            SessionStreamingPayload {
                                session_id,
                                streaming: false,
                            },
                        );
                    }
                    // Don't break — process is still alive, waiting for next input
                }
            }
        }
    })
}

/// Spawn a tokio task that reads stderr and emits error events.
pub fn spawn_stderr_reader(
    session_id: Uuid,
    stderr: ChildStderr,
    app: tauri::AppHandle,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(raw_line)) = lines.next_line().await {
            let line = strip_ansi_codes(&raw_line);
            if !line.trim().is_empty() {
                let _ = app.emit(
                    "session:error",
                    SessionErrorPayload {
                        session_id,
                        error: line,
                    },
                );
            }
        }
    })
}

// ── Stream-JSON reader (for --print --output-format stream-json) ────────────

/// Spawn a tokio task that reads stream-json output from a per-message Claude
/// process, parses events, emits Tauri events, and extracts the Claude CLI
/// session ID for `--resume`.
///
/// The `_child_guard` ensures the child process is killed if this task is
/// aborted or dropped.
pub fn spawn_json_reader(
    session_id: Uuid,
    stdout: ChildStdout,
    _child_guard: ChildGuard,
    claude_session_id: Arc<AsyncMutex<Option<String>>>,
    app: tauri::AppHandle,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let _guard = _child_guard;
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        // Emit streaming indicator — the process is running
        let _ = app.emit(
            "session:streaming",
            SessionStreamingPayload {
                session_id,
                streaming: true,
            },
        );

        while let Ok(Some(line)) = lines.next_line().await {
            if line.trim().is_empty() {
                continue;
            }

            if let Some(event) = parse_stream_json(&line) {
                match event {
                    StreamEvent::SessionInit {
                        session_id: sid, ..
                    } => {
                        *claude_session_id.lock().await = Some(sid);
                    }
                    StreamEvent::TextDelta { text } => {
                        let _ = app.emit(
                            "session:output",
                            SessionOutputPayload {
                                session_id,
                                text,
                            },
                        );
                    }
                    StreamEvent::MessageStop => {
                        // Message complete — no action needed, streaming indicator
                        // will be cleared when the process exits (below).
                    }
                    StreamEvent::Result {
                        session_id: sid,
                        is_error,
                        error_text,
                    } => {
                        if let Some(sid) = sid {
                            *claude_session_id.lock().await = Some(sid);
                        }
                        if is_error {
                            let _ = app.emit(
                                "session:error",
                                SessionErrorPayload {
                                    session_id,
                                    error: error_text
                                        .unwrap_or_else(|| "Unknown error".to_string()),
                                },
                            );
                        }
                    }
                }
            }
        }

        // Process exited — streaming is done
        let _ = app.emit(
            "session:streaming",
            SessionStreamingPayload {
                session_id,
                streaming: false,
            },
        );
    })
}

// ── Orchestration event payloads ─────────────────────────────────────────────

#[derive(Clone, Serialize)]
pub struct AgentEventPayload {
    pub session_id: Uuid,
    pub agent_type: String,
    pub task: String,
}

#[derive(Clone, Serialize)]
pub struct PhaseChangePayload {
    pub session_id: Uuid,
    pub phase: u32,
    pub name: String,
}

fn emit_claude_event(app: &tauri::AppHandle, session_id: Uuid, event: &ClaudeEvent) {
    match event {
        ClaudeEvent::TextOutput { text } => {
            let _ = app.emit(
                "session:output",
                SessionOutputPayload {
                    session_id,
                    text: text.clone(),
                },
            );
        }
        ClaudeEvent::Question {
            text,
            suggested_answers,
        } => {
            let _ = app.emit(
                "session:question",
                SessionQuestionPayload {
                    session_id,
                    text: text.clone(),
                    suggested_answers: suggested_answers.clone(),
                },
            );
        }
        ClaudeEvent::ToolUse {
            tool,
            input_summary,
        } => {
            let _ = app.emit(
                "session:tool-use",
                ToolUsePayload {
                    session_id,
                    tool: tool.clone(),
                    input_summary: input_summary.clone(),
                },
            );
        }
        ClaudeEvent::PermissionRequest { tool, description } => {
            let _ = app.emit(
                "session:permission-request",
                PermissionRequestPayload {
                    session_id,
                    tool: tool.clone(),
                    description: description.clone(),
                },
            );
        }
        ClaudeEvent::AgentSpawned { agent_type, task } => {
            let _ = app.emit(
                "orchestration:agent-spawned",
                AgentEventPayload {
                    session_id,
                    agent_type: agent_type.to_string(),
                    task: task.clone(),
                },
            );
        }
        ClaudeEvent::AgentCompleted { agent_type, result } => {
            let _ = app.emit(
                "orchestration:agent-completed",
                AgentEventPayload {
                    session_id,
                    agent_type: agent_type.to_string(),
                    task: result.clone(),
                },
            );
        }
        ClaudeEvent::PhaseChange { phase, name } => {
            let _ = app.emit(
                "orchestration:phase-change",
                PhaseChangePayload {
                    session_id,
                    phase: *phase,
                    name: name.clone(),
                },
            );
        }
        ClaudeEvent::FileChanged { path, action } => {
            let _ = app.emit(
                "orchestration:file-changed",
                SessionOutputPayload {
                    session_id,
                    text: format!("{}: {}", action, path),
                },
            );
        }
        _ => {}
    }
}
