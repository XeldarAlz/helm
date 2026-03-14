use std::sync::OnceLock;

use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::models::pipeline::AgentType;

/// Structured events parsed from Claude Code's stdout stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClaudeEvent {
    TextOutput { text: String },
    Question { text: String, suggested_answers: Vec<String> },
    ToolUse { tool: String, input_summary: String },
    PermissionRequest { tool: String, description: String },
    AgentSpawned { agent_type: AgentType, task: String },
    AgentCompleted { agent_type: AgentType, result: String },
    FileChanged { path: String, action: String },
    PhaseChange { phase: u32, name: String },
    Streaming,
    ResponseComplete,
}

// ── ANSI escape code stripping ───────────────────────────────────────────────

fn ansi_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        // Matches CSI sequences (\e[...X), OSC sequences (\e]...\a), and
        // charset selection sequences (\e(X / \e)X).
        Regex::new(
            r"(?x)
            \x1b \[ [0-9;]* [A-Za-z]    |   # CSI sequences (colors, cursor, etc.)
            \x1b \] [^\x07]* \x07        |   # OSC sequences (title, hyperlink, etc.)
            \x1b [()]  [AB012]               # Charset selection
            "
        ).unwrap()
    })
}

/// Strip ANSI escape codes from a string.  Called in the I/O bridge before
/// lines reach the parser so that neither the parser nor the frontend ever
/// sees raw escape sequences.
pub fn strip_ansi_codes(s: &str) -> String {
    ansi_regex().replace_all(s, "").to_string()
}

// ── Permission prompt detection ──────────────────────────────────────────────

fn permission_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        // Claude Code permission prompts follow patterns like:
        //   "Allow Claude to use Bash?"
        //   "Allow tool Read?"
        //   "Do you want to allow Edit?"
        Regex::new(
            r"(?i)(?:allow|permit|approve)\s+(?:claude\s+to\s+)?(?:use\s+)?(\w+)\s*\?",
        )
        .unwrap()
    })
}

// ── Line-based output parser ─────────────────────────────────────────────────

/// Line-based parser that converts raw stdout text into structured ClaudeEvents.
/// Handles text output, question detection with suggested answers, and
/// permission prompt detection.
pub struct OutputParser {
    /// Buffered suggested answers following a detected question
    answer_buffer: Vec<String>,
    /// Pending question text waiting for potential suggested answers
    pending_question: Option<String>,
}

impl OutputParser {
    pub fn new() -> Self {
        Self {
            answer_buffer: Vec::new(),
            pending_question: None,
        }
    }

    /// Parse a single line of stdout output into zero or more events.
    /// The caller is expected to strip ANSI codes before calling this.
    pub fn parse_line(&mut self, line: &str) -> Vec<ClaudeEvent> {
        let trimmed = line.trim();

        // Empty line: if we have a pending question, flush it
        if trimmed.is_empty() {
            return self.flush_pending();
        }

        // If we have a pending question, check for numbered/bulleted options
        if self.pending_question.is_some() {
            if let Some(answer) = Self::extract_option(trimmed) {
                self.answer_buffer.push(answer);
                return vec![];
            }
            // Not an option — flush the question, then process this line
            let mut events = self.flush_pending();
            events.extend(self.classify_line(line, trimmed));
            return events;
        }

        // Check if this line is a question
        if trimmed.ends_with('?') {
            // Check for a permission prompt first
            if let Some(caps) = permission_regex().captures(trimmed) {
                let tool = caps.get(1).map(|m| m.as_str()).unwrap_or("unknown");
                return vec![ClaudeEvent::PermissionRequest {
                    tool: tool.to_string(),
                    description: trimmed.to_string(),
                }];
            }

            self.pending_question = Some(trimmed.to_string());
            self.answer_buffer.clear();
            return vec![];
        }

        self.classify_line(line, trimmed)
    }

    /// Flush any buffered state (call on stream end or timeout).
    pub fn flush(&mut self) -> Vec<ClaudeEvent> {
        self.flush_pending()
    }

    fn classify_line(&self, line: &str, _trimmed: &str) -> Vec<ClaudeEvent> {
        vec![ClaudeEvent::TextOutput {
            text: line.to_string(),
        }]
    }

    fn flush_pending(&mut self) -> Vec<ClaudeEvent> {
        if let Some(question_text) = self.pending_question.take() {
            let suggested = std::mem::take(&mut self.answer_buffer);
            vec![ClaudeEvent::Question {
                text: question_text,
                suggested_answers: suggested,
            }]
        } else {
            vec![]
        }
    }

    /// Try to extract a suggested answer from a numbered or bulleted line.
    /// Matches: "1. Foo", "2) Bar", "- Baz", as well as multi-digit numbers.
    fn extract_option(line: &str) -> Option<String> {
        let trimmed = line.trim();

        // Bullet: "- text"
        if let Some(rest) = trimmed.strip_prefix("- ") {
            return Some(rest.to_string());
        }

        // Numbered: "N. text" or "N) text"  (one or more digits)
        if let Some(first) = trimmed.chars().next() {
            if first.is_ascii_digit() {
                // Find the end of the digit run
                let digit_end = trimmed
                    .chars()
                    .position(|c| !c.is_ascii_digit())
                    .unwrap_or(trimmed.len());
                let rest = &trimmed[digit_end..];
                if let Some(text) = rest.strip_prefix(". ") {
                    return Some(text.to_string());
                }
                if let Some(text) = rest.strip_prefix(") ") {
                    return Some(text.to_string());
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_emits_text_output() {
        let mut p = OutputParser::new();
        let events = p.parse_line("Hello world");
        assert_eq!(events.len(), 1);
        match &events[0] {
            ClaudeEvent::TextOutput { text } => assert_eq!(text, "Hello world"),
            _ => panic!("expected TextOutput"),
        }
    }

    #[test]
    fn question_detected_on_flush() {
        let mut p = OutputParser::new();
        assert!(p.parse_line("What genre would you like?").is_empty());
        let events = p.flush();
        assert_eq!(events.len(), 1);
        match &events[0] {
            ClaudeEvent::Question { text, suggested_answers } => {
                assert_eq!(text, "What genre would you like?");
                assert!(suggested_answers.is_empty());
            }
            _ => panic!("expected Question"),
        }
    }

    #[test]
    fn question_with_numbered_answers() {
        let mut p = OutputParser::new();
        assert!(p.parse_line("What genre?").is_empty());
        assert!(p.parse_line("1. RPG").is_empty());
        assert!(p.parse_line("2. Platformer").is_empty());
        assert!(p.parse_line("3. Puzzle").is_empty());
        let events = p.parse_line("");
        assert_eq!(events.len(), 1);
        match &events[0] {
            ClaudeEvent::Question { text, suggested_answers } => {
                assert_eq!(text, "What genre?");
                assert_eq!(suggested_answers, &["RPG", "Platformer", "Puzzle"]);
            }
            _ => panic!("expected Question"),
        }
    }

    #[test]
    fn question_with_bullet_answers() {
        let mut p = OutputParser::new();
        assert!(p.parse_line("Pick one?").is_empty());
        assert!(p.parse_line("- Alpha").is_empty());
        assert!(p.parse_line("- Beta").is_empty());
        let events = p.flush();
        assert_eq!(events.len(), 1);
        match &events[0] {
            ClaudeEvent::Question { text, suggested_answers } => {
                assert_eq!(text, "Pick one?");
                assert_eq!(suggested_answers, &["Alpha", "Beta"]);
            }
            _ => panic!("expected Question"),
        }
    }

    #[test]
    fn question_followed_by_non_option_flushes_question_then_text() {
        let mut p = OutputParser::new();
        assert!(p.parse_line("Ready?").is_empty());
        let events = p.parse_line("Some regular text");
        assert_eq!(events.len(), 2);
        assert!(matches!(&events[0], ClaudeEvent::Question { .. }));
        assert!(matches!(&events[1], ClaudeEvent::TextOutput { .. }));
    }

    #[test]
    fn empty_lines_are_no_ops() {
        let mut p = OutputParser::new();
        let events = p.parse_line("");
        assert!(events.is_empty());
    }

    #[test]
    fn multi_digit_numbered_options() {
        let mut p = OutputParser::new();
        assert!(p.parse_line("Choose?").is_empty());
        assert!(p.parse_line("10. Option ten").is_empty());
        assert!(p.parse_line("11. Option eleven").is_empty());
        let events = p.flush();
        assert_eq!(events.len(), 1);
        match &events[0] {
            ClaudeEvent::Question { suggested_answers, .. } => {
                assert_eq!(suggested_answers, &["Option ten", "Option eleven"]);
            }
            _ => panic!("expected Question"),
        }
    }

    #[test]
    fn strip_ansi_codes_removes_color_sequences() {
        let input = "\x1b[32mHello\x1b[0m world";
        assert_eq!(strip_ansi_codes(input), "Hello world");
    }

    #[test]
    fn strip_ansi_codes_removes_osc_sequences() {
        let input = "\x1b]0;Window Title\x07Some text";
        assert_eq!(strip_ansi_codes(input), "Some text");
    }

    #[test]
    fn strip_ansi_codes_noop_on_clean_text() {
        let input = "Hello world";
        assert_eq!(strip_ansi_codes(input), "Hello world");
    }

    #[test]
    fn permission_prompt_detected() {
        let mut p = OutputParser::new();
        let events = p.parse_line("Allow Claude to use Bash?");
        assert_eq!(events.len(), 1);
        match &events[0] {
            ClaudeEvent::PermissionRequest { tool, description } => {
                assert_eq!(tool, "Bash");
                assert_eq!(description, "Allow Claude to use Bash?");
            }
            _ => panic!("expected PermissionRequest"),
        }
    }

    #[test]
    fn regular_question_not_matched_as_permission() {
        let mut p = OutputParser::new();
        let events = p.parse_line("What genre would you like?");
        // Should be buffered as a question, not a permission prompt
        assert!(events.is_empty());
        let events = p.flush();
        assert!(matches!(&events[0], ClaudeEvent::Question { .. }));
    }
}

// ── Stream-JSON parser (for --output-format stream-json) ────────────────────

/// Events parsed from Claude Code's `--output-format stream-json` output.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// Session init — contains the Claude CLI session ID for `--resume`.
    SessionInit { session_id: String },
    /// A text delta from the streaming response (token-level).
    TextDelta { text: String },
    /// The assistant's streaming message is complete.
    MessageStop,
    /// Final result — the process will exit shortly after.
    Result {
        session_id: Option<String>,
        is_error: bool,
        error_text: Option<String>,
    },
}

/// Parse a single line of stream-json output into an optional event.
pub fn parse_stream_json(line: &str) -> Option<StreamEvent> {
    let v: serde_json::Value = serde_json::from_str(line).ok()?;
    let event_type = v.get("type")?.as_str()?;

    match event_type {
        "system" => {
            let session_id = v.get("session_id")?.as_str()?.to_string();
            Some(StreamEvent::SessionInit { session_id })
        }
        "stream_event" => {
            let inner = v.get("event")?;
            let inner_type = inner.get("type")?.as_str()?;
            match inner_type {
                "content_block_delta" => {
                    let delta = inner.get("delta")?;
                    if delta.get("type")?.as_str()? == "text_delta" {
                        let text = delta.get("text")?.as_str()?.to_string();
                        Some(StreamEvent::TextDelta { text })
                    } else {
                        None
                    }
                }
                "message_stop" => Some(StreamEvent::MessageStop),
                _ => None,
            }
        }
        "result" => {
            let session_id = v
                .get("session_id")
                .and_then(|s| s.as_str())
                .map(|s| s.to_string());
            let is_error = v
                .get("is_error")
                .and_then(|e| e.as_bool())
                .unwrap_or(false);
            let error_text = if is_error {
                v.get("result")
                    .and_then(|r| r.as_str())
                    .map(|s| s.to_string())
            } else {
                None
            };
            Some(StreamEvent::Result {
                session_id,
                is_error,
                error_text,
            })
        }
        _ => None,
    }
}

#[cfg(test)]
mod stream_json_tests {
    use super::*;

    #[test]
    fn parse_system_init() {
        let line = r#"{"type":"system","subtype":"init","session_id":"abc-123","cwd":"/tmp"}"#;
        match parse_stream_json(line) {
            Some(StreamEvent::SessionInit { session_id }) => {
                assert_eq!(session_id, "abc-123");
            }
            other => panic!("expected SessionInit, got {:?}", other),
        }
    }

    #[test]
    fn parse_text_delta() {
        let line = r#"{"type":"stream_event","event":{"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"hello"}}}"#;
        match parse_stream_json(line) {
            Some(StreamEvent::TextDelta { text }) => assert_eq!(text, "hello"),
            other => panic!("expected TextDelta, got {:?}", other),
        }
    }

    #[test]
    fn parse_message_stop() {
        let line = r#"{"type":"stream_event","event":{"type":"message_stop"}}"#;
        assert!(matches!(
            parse_stream_json(line),
            Some(StreamEvent::MessageStop)
        ));
    }

    #[test]
    fn parse_result_success() {
        let line = r#"{"type":"result","subtype":"success","is_error":false,"result":"pong","session_id":"xyz-789"}"#;
        match parse_stream_json(line) {
            Some(StreamEvent::Result {
                session_id,
                is_error,
                error_text,
            }) => {
                assert_eq!(session_id.as_deref(), Some("xyz-789"));
                assert!(!is_error);
                assert!(error_text.is_none());
            }
            other => panic!("expected Result, got {:?}", other),
        }
    }

    #[test]
    fn parse_result_error() {
        let line =
            r#"{"type":"result","subtype":"error","is_error":true,"result":"API key invalid"}"#;
        match parse_stream_json(line) {
            Some(StreamEvent::Result {
                is_error,
                error_text,
                ..
            }) => {
                assert!(is_error);
                assert_eq!(error_text.as_deref(), Some("API key invalid"));
            }
            other => panic!("expected Result error, got {:?}", other),
        }
    }

    #[test]
    fn unknown_event_returns_none() {
        let line = r#"{"type":"rate_limit_event","rate_limit_info":{}}"#;
        assert!(parse_stream_json(line).is_none());
    }

    #[test]
    fn invalid_json_returns_none() {
        assert!(parse_stream_json("not json").is_none());
    }
}
