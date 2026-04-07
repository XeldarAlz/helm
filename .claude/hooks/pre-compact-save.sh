#!/bin/bash
# pre-compact-save.sh — PreCompact hook
# Consolidates agent state into a recovery brief before context compaction.
# Non-blocking (always exit 0). Writes .claude/pre-compact-state.md.

INPUT=$(cat)
CWD=$(echo "$INPUT" | jq -r '.cwd // empty' 2>/dev/null)
if [ -z "$CWD" ]; then
    CWD="$(pwd)"
fi

CHECKPOINT_DIR="$CWD/.claude/checkpoint"
MAILBOX_DIR="$CWD/.claude/mailbox"
PROGRESS="$CWD/docs/PROGRESS.md"
EVENTS="$CWD/docs/EVENTS.jsonl"
OUTPUT="$CWD/.claude/pre-compact-state.md"
TIMESTAMP=$(date -u +%Y-%m-%dT%H:%M:%SZ)

# Start building the recovery brief
{
    echo "# Pre-Compact Recovery Brief"
    echo "## Saved: $TIMESTAMP"
    echo ""

    # Pipeline state from PROGRESS.md
    echo "## Pipeline State"
    if [ -f "$PROGRESS" ]; then
        sed -n 's/^## Status: /- **Status:** /p' "$PROGRESS" | head -1
        sed -n 's/^## Phase: /- **Phase:** /p' "$PROGRESS" | head -1
        sed -n 's/^## Phase Name: /- **Phase Name:** /p' "$PROGRESS" | head -1
        sed -n 's/^## Started: /- **Started:** /p' "$PROGRESS" | head -1
    else
        echo "No PROGRESS.md found."
    fi
    echo ""

    # Recent events
    echo "## Recent Events (last 20)"
    if [ -f "$EVENTS" ]; then
        echo '```jsonl'
        tail -20 "$EVENTS"
        echo '```'
    else
        echo "No EVENTS.jsonl found."
    fi
    echo ""

    # Agent checkpoints
    echo "## Agent Checkpoints"
    if [ -d "$CHECKPOINT_DIR" ]; then
        FOUND_CHECKPOINTS=0
        for checkpoint in "$CHECKPOINT_DIR"/*.md; do
            [ -f "$checkpoint" ] || continue
            FOUND_CHECKPOINTS=1
            AGENT_NAME=$(basename "$checkpoint" .md)
            echo "### $AGENT_NAME"
            cat "$checkpoint"
            echo ""
        done
        if [ "$FOUND_CHECKPOINTS" -eq 0 ]; then
            echo "No checkpoint files found."
        fi
    else
        echo "No checkpoint directory found."
    fi
    echo ""

    # Recent mailbox activity
    echo "## Recent Mailbox Activity"
    if [ -d "$MAILBOX_DIR" ]; then
        FOUND_MAILBOX=0
        for mailbox in "$MAILBOX_DIR"/*.jsonl; do
            [ -f "$mailbox" ] || continue
            FOUND_MAILBOX=1
            AGENT_NAME=$(basename "$mailbox" .jsonl)
            echo "### $AGENT_NAME (last 5 messages)"
            echo '```jsonl'
            tail -5 "$mailbox"
            echo '```'
            echo ""
        done
        if [ "$FOUND_MAILBOX" -eq 0 ]; then
            echo "No mailbox files found."
        fi
    else
        echo "No mailbox directory found."
    fi
    echo ""

    # Recovery instructions
    echo "## Recovery Instructions"
    echo "After compaction, read this file to restore context. Cross-reference with"
    echo "docs/PROGRESS.md and docs/EVENTS.jsonl for authoritative task states."
    echo "Individual agent checkpoints in .claude/checkpoint/ have per-agent detail."

} > "$OUTPUT"

echo "Pre-compact state saved to .claude/pre-compact-state.md" >&2

exit 0
