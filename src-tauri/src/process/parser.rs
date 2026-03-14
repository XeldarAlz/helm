use serde::{Deserialize, Serialize};

use crate::models::pipeline::AgentType;

/// Structured events parsed from Claude Code's stdout stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClaudeEvent {
    TextOutput { text: String },
    Question { text: String, suggested_answers: Vec<String> },
    AgentSpawned { agent_type: AgentType, task: String },
    AgentCompleted { agent_type: AgentType, result: String },
    FileChanged { path: String, action: String },
    PhaseChange { phase: u32, name: String },
    Streaming,
    ResponseComplete,
}

/// Line-based parser that converts raw stdout text into structured ClaudeEvents.
/// Phase 2: handles text output and basic question detection with suggested answers.
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
    /// Matches: "1. Foo", "2) Bar", "- Baz"
    fn extract_option(line: &str) -> Option<String> {
        let trimmed = line.trim();

        // Bullet: "- text"
        if let Some(rest) = trimmed.strip_prefix("- ") {
            return Some(rest.to_string());
        }

        // Numbered: "N. text" or "N) text"
        let mut chars = trimmed.chars();
        if let Some(first) = chars.next() {
            if first.is_ascii_digit() {
                let rest: String = chars.collect();
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
}
