use std::time::Duration;

use serde::Serialize;
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{ChildStderr, ChildStdout};
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::process::parser::{ClaudeEvent, OutputParser};

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

/// How long to wait for the next line before declaring a response complete.
const RESPONSE_TIMEOUT: Duration = Duration::from_millis(1500);

/// Spawn a tokio task that reads the child process stdout line-by-line,
/// parses it with OutputParser, and emits Tauri events to the frontend.
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
                Ok(Ok(Some(line))) => {
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
        while let Ok(Some(line)) = lines.next_line().await {
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

// ── Phase 6: Orchestration event payloads ────────────────────────────────────

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
