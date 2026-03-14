//! Integration test: spawn a process, pipe stdin/stdout, parse output.
//! Uses `cat` as a mock Claude CLI — it echoes stdin back to stdout,
//! letting us verify the full data flow without requiring `claude` installed.

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

use helm_lib::process::parser::{ClaudeEvent, OutputParser};

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Spawn `cat` with piped stdin/stdout and return handles.
async fn spawn_mock() -> (
    tokio::process::ChildStdin,
    BufReader<tokio::process::ChildStdout>,
    tokio::process::Child,
) {
    let mut child = Command::new("cat")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn cat");

    let stdin = child.stdin.take().unwrap();
    let stdout = BufReader::new(child.stdout.take().unwrap());
    (stdin, stdout, child)
}

/// Write a line to stdin and read the echo from stdout.
async fn send_and_read(
    stdin: &mut tokio::process::ChildStdin,
    reader: &mut BufReader<tokio::process::ChildStdout>,
    msg: &str,
) -> String {
    stdin
        .write_all(format!("{}\n", msg).as_bytes())
        .await
        .unwrap();
    stdin.flush().await.unwrap();

    let mut line = String::new();
    reader.read_line(&mut line).await.unwrap();
    line
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn spawn_send_receive_plain_text() {
    let (mut stdin, mut reader, mut child) = spawn_mock().await;

    let line = send_and_read(&mut stdin, &mut reader, "Hello from Helm").await;
    assert_eq!(line.trim(), "Hello from Helm");

    // Parse through OutputParser
    let mut parser = OutputParser::new();
    let events = parser.parse_line(line.trim());

    assert_eq!(events.len(), 1);
    match &events[0] {
        ClaudeEvent::TextOutput { text } => assert_eq!(text, "Hello from Helm"),
        other => panic!("expected TextOutput, got {:?}", other),
    }

    child.kill().await.unwrap();
}

#[tokio::test]
async fn question_with_suggested_answers() {
    let (mut stdin, mut reader, mut child) = spawn_mock().await;
    let mut parser = OutputParser::new();

    // Send a question — parser buffers it
    let q = send_and_read(&mut stdin, &mut reader, "What genre?").await;
    let events = parser.parse_line(q.trim());
    assert!(events.is_empty(), "question should be buffered");

    // Send numbered options — parser collects them
    let o1 = send_and_read(&mut stdin, &mut reader, "1. RPG").await;
    assert!(parser.parse_line(o1.trim()).is_empty());

    let o2 = send_and_read(&mut stdin, &mut reader, "2. Platformer").await;
    assert!(parser.parse_line(o2.trim()).is_empty());

    let o3 = send_and_read(&mut stdin, &mut reader, "3. Puzzle").await;
    assert!(parser.parse_line(o3.trim()).is_empty());

    // Empty line flushes the question
    let blank = send_and_read(&mut stdin, &mut reader, "").await;
    let events = parser.parse_line(blank.trim());

    assert_eq!(events.len(), 1);
    match &events[0] {
        ClaudeEvent::Question {
            text,
            suggested_answers,
        } => {
            assert_eq!(text, "What genre?");
            assert_eq!(suggested_answers, &["RPG", "Platformer", "Puzzle"]);
        }
        other => panic!("expected Question, got {:?}", other),
    }

    child.kill().await.unwrap();
}

#[tokio::test]
async fn multiple_messages_in_sequence() {
    let (mut stdin, mut reader, mut child) = spawn_mock().await;
    let mut parser = OutputParser::new();

    // Send several messages and verify each is received and parsed
    for msg in ["First message", "Second message", "Third message"] {
        let line = send_and_read(&mut stdin, &mut reader, msg).await;
        let events = parser.parse_line(line.trim());

        assert_eq!(events.len(), 1);
        match &events[0] {
            ClaudeEvent::TextOutput { text } => assert_eq!(text, msg),
            other => panic!("expected TextOutput, got {:?}", other),
        }
    }

    child.kill().await.unwrap();
}

#[tokio::test]
async fn flush_on_process_kill() {
    let (mut stdin, mut reader, mut child) = spawn_mock().await;
    let mut parser = OutputParser::new();

    // Buffer a question without flushing
    let q = send_and_read(&mut stdin, &mut reader, "Ready?").await;
    assert!(parser.parse_line(q.trim()).is_empty());

    // Simulate process exit — flush parser
    child.kill().await.unwrap();

    let remaining = parser.flush();
    assert_eq!(remaining.len(), 1);
    match &remaining[0] {
        ClaudeEvent::Question {
            text,
            suggested_answers,
        } => {
            assert_eq!(text, "Ready?");
            assert!(suggested_answers.is_empty());
        }
        other => panic!("expected Question, got {:?}", other),
    }
}

#[tokio::test]
async fn stdin_write_after_kill_fails_gracefully() {
    let (mut stdin, _reader, mut child) = spawn_mock().await;

    child.kill().await.unwrap();

    // Writing to a dead process stdin should error, not panic
    let result = stdin.write_all(b"hello\n").await;
    // The write may or may not fail depending on OS buffering,
    // but it must not panic
    drop(result);
}

#[tokio::test]
async fn bullet_style_suggested_answers() {
    let (mut stdin, mut reader, mut child) = spawn_mock().await;
    let mut parser = OutputParser::new();

    let q = send_and_read(&mut stdin, &mut reader, "Pick a style?").await;
    assert!(parser.parse_line(q.trim()).is_empty());

    let b1 = send_and_read(&mut stdin, &mut reader, "- Pixel art").await;
    assert!(parser.parse_line(b1.trim()).is_empty());

    let b2 = send_and_read(&mut stdin, &mut reader, "- 3D low-poly").await;
    assert!(parser.parse_line(b2.trim()).is_empty());

    // Flush via explicit call (simulates timeout)
    let events = parser.flush();
    assert_eq!(events.len(), 1);
    match &events[0] {
        ClaudeEvent::Question {
            text,
            suggested_answers,
        } => {
            assert_eq!(text, "Pick a style?");
            assert_eq!(suggested_answers, &["Pixel art", "3D low-poly"]);
        }
        other => panic!("expected Question, got {:?}", other),
    }

    child.kill().await.unwrap();
}
