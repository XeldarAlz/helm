use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{ChildStderr, ChildStdout};
use tokio::sync::Mutex as AsyncMutex;
use tokio::task::JoinHandle;
use uuid::Uuid;

use crate::models::pipeline::{LogEntry, LogLevel, OrchLiveLog};
use crate::process::manager::ChildGuard;
use crate::process::parser::{
    detect_orchestration_pattern, parse_stream_json, strip_ansi_codes, ClaudeEvent, OutputParser,
    StreamEvent,
};

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

        // Buffer for accumulating text deltas into complete lines for
        // orchestration pattern detection.
        let mut orch_text_buf = String::new();

        // Emit streaming indicator — the process is running
        if let Err(e) = app.emit(
            "session:streaming",
            SessionStreamingPayload {
                session_id,
                streaming: true,
            },
        ) {
            eprintln!("[helm:json-reader] FAILED to emit streaming:true — {e}");
        }

        let mut line_count: u32 = 0;
        let mut text_events: u32 = 0;
        let mut result_text: Option<String> = None;
        while let Ok(Some(line)) = lines.next_line().await {
            line_count += 1;
            if line.trim().is_empty() {
                continue;
            }

            if let Some(event) = parse_stream_json(&line) {
                match event {
                    StreamEvent::SessionInit {
                        session_id: sid, ..
                    } => {
                        eprintln!("[helm:json-reader] init — claude session {sid}");
                        *claude_session_id.lock().await = Some(sid);
                    }
                    StreamEvent::TextDelta { text } => {
                        text_events += 1;

                        // ── Orchestration pattern detection ─────────────
                        // Accumulate text deltas into a line buffer and
                        // check each complete line for orchestration events.
                        orch_text_buf.push_str(&text);
                        while let Some(nl) = orch_text_buf.find('\n') {
                            let complete_line: String =
                                orch_text_buf.drain(..=nl).collect();
                            if let Some(orch_event) =
                                detect_orchestration_pattern(&complete_line)
                            {
                                // Emit the event to the frontend
                                emit_claude_event(&app, session_id, &orch_event);
                                // Also persist in the live-log so
                                // get_orchestration_state can include it
                                record_live_event(&app, &orch_event);
                            }
                        }

                        if let Err(e) = app.emit(
                            "session:output",
                            SessionOutputPayload {
                                session_id,
                                text,
                            },
                        ) {
                            eprintln!("[helm:json-reader] FAILED to emit output — {e}");
                        }
                    }
                    StreamEvent::MessageStop => {
                        // Flush remaining text buffer
                        if !orch_text_buf.is_empty() {
                            let remaining = std::mem::take(&mut orch_text_buf);
                            if let Some(orch_event) =
                                detect_orchestration_pattern(&remaining)
                            {
                                emit_claude_event(&app, session_id, &orch_event);
                                record_live_event(&app, &orch_event);
                            }
                        }
                    }
                    StreamEvent::Result {
                        session_id: sid,
                        is_error,
                        error_text,
                        result_text: res_text,
                    } => {
                        if let Some(sid) = sid {
                            *claude_session_id.lock().await = Some(sid);
                        }
                        if is_error {
                            let err = error_text
                                .unwrap_or_else(|| "Unknown error".to_string());
                            eprintln!("[helm:json-reader] result error — {err}");
                            let _ = app.emit(
                                "session:error",
                                SessionErrorPayload {
                                    session_id,
                                    error: err,
                                },
                            );
                        }
                        result_text = res_text;
                    }
                }
            }
        }

        // If the process exited with zero text output but has a result message,
        // surface it so the user isn't left staring at an empty screen.
        if text_events == 0 {
            if let Some(msg) = result_text {
                eprintln!("[helm:json-reader] no text output — surfacing result: {msg}");
                let _ = app.emit(
                    "session:error",
                    SessionErrorPayload {
                        session_id,
                        error: msg,
                    },
                );
            }
        }

        // Process exited — streaming is done
        eprintln!(
            "[helm:json-reader] done — {line_count} lines, {text_events} text deltas"
        );
        let _ = app.emit(
            "session:streaming",
            SessionStreamingPayload {
                session_id,
                streaming: false,
            },
        );
    })
}

/// Record an orchestration event in the in-memory live log.
fn record_live_event(app: &tauri::AppHandle, event: &ClaudeEvent) {
    use tauri::Manager;
    let Some(log) = app.try_state::<OrchLiveLog>() else {
        return;
    };
    let now = chrono::Utc::now().to_rfc3339();

    match event {
        ClaudeEvent::AgentSpawned { agent_type, task } => {
            log.set_running();
            log.push(LogEntry {
                timestamp: now,
                level: LogLevel::Agent,
                source: format!("agent:{}", agent_type),
                message: format!("Spawned: {}", task),
            });
        }
        ClaudeEvent::AgentCompleted { agent_type, result } => {
            log.push(LogEntry {
                timestamp: now,
                level: LogLevel::Agent,
                source: format!("agent:{}", agent_type),
                message: format!("Completed: {}", result),
            });
        }
        ClaudeEvent::PhaseChange { phase, name } => {
            log.set_running();
            log.set_phase(*phase);
            log.push(LogEntry {
                timestamp: now,
                level: LogLevel::System,
                source: "system".to_string(),
                message: format!("Phase {}: {}", phase, name),
            });
        }
        _ => {}
    }
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
