#!/bin/bash
# checkpoint-nudge.sh — PostToolUse hook
# Nudges agents to update their checkpoint file after many writes.
# Non-blocking (always exit 0). Prints reminder to stderr.

# Only check .cs files (agent implementation work)
INPUT=$(cat)
FILE_PATH=$(echo "$INPUT" | grep -o '"file_path":"[^"]*"' | head -1 | cut -d'"' -f4)

if [[ -z "$FILE_PATH" ]]; then
  exit 0
fi

# Only care about C# files in the project (not docs, not .claude)
if [[ "$FILE_PATH" != *.cs ]]; then
  exit 0
fi

if [[ "$FILE_PATH" == *".claude/"* ]] || [[ "$FILE_PATH" == *"docs/"* ]]; then
  exit 0
fi

# Count recent writes in ACTIVITY_LOG.md since last checkpoint update
ACTIVITY_LOG="docs/ACTIVITY_LOG.md"
CHECKPOINT_DIR=".claude/checkpoint"

if [[ ! -f "$ACTIVITY_LOG" ]]; then
  exit 0
fi

# Count .cs file entries in activity log
CS_WRITES=$(grep -c '\.cs' "$ACTIVITY_LOG" 2>/dev/null || echo "0")

# Check if any checkpoint file exists and was recently updated
LATEST_CHECKPOINT=""
if [[ -d "$CHECKPOINT_DIR" ]]; then
  LATEST_CHECKPOINT=$(find "$CHECKPOINT_DIR" -name "*.md" -newer "$ACTIVITY_LOG" 2>/dev/null | head -1)
fi

# If we have more than 8 .cs writes and no recent checkpoint, nudge
if [[ "$CS_WRITES" -gt 8 ]] && [[ -z "$LATEST_CHECKPOINT" ]]; then
  echo "CHECKPOINT REMINDER: You have written $CS_WRITES files since your last checkpoint. Consider updating your checkpoint file at .claude/checkpoint/<your-agent-id>.md with your current progress, completed work, and key decisions. This protects against context loss." >&2
fi

exit 0
