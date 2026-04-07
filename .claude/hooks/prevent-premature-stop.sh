#!/bin/bash
# prevent-premature-stop.sh — Stop hook (BLOCKING)
# Prevents Claude from stopping prematurely during active orchestration.
# Exit 2 = block stop, exit 0 = allow stop.

INPUT=$(cat)
CWD=$(echo "$INPUT" | jq -r '.cwd // empty' 2>/dev/null)
if [ -z "$CWD" ]; then
    CWD="$(pwd)"
fi

ACTIVE_FILE="$CWD/.claude/orchestration-active.json"

# No active marker = allow stop
if [ ! -f "$ACTIVE_FILE" ]; then
    exit 0
fi

# Check file age (macOS stat)
MTIME=$(stat -f %m "$ACTIVE_FILE" 2>/dev/null)
if [ -z "$MTIME" ]; then
    # Fallback for GNU stat
    MTIME=$(stat -c %Y "$ACTIVE_FILE" 2>/dev/null)
fi

if [ -z "$MTIME" ]; then
    # Can't determine age, allow stop
    exit 0
fi

NOW=$(date +%s)
AGE=$(( NOW - MTIME ))
MAX_AGE=7200  # 2 hours

# Stale marker = clean up and allow stop
if [ "$AGE" -gt "$MAX_AGE" ]; then
    rm -f "$ACTIVE_FILE"
    echo "Stale orchestration marker removed (>2h old). Stop allowed." >&2
    exit 0
fi

# Active orchestration — read context for the blocking message
PHASE=$(jq -r '.phase // "unknown"' "$ACTIVE_FILE" 2>/dev/null)
PHASE_NAME=$(jq -r '.phaseName // "unknown"' "$ACTIVE_FILE" 2>/dev/null)
STARTED=$(jq -r '.started // "unknown"' "$ACTIVE_FILE" 2>/dev/null)

echo "BLOCKED: Orchestration is active (started: $STARTED, phase $PHASE: $PHASE_NAME)." >&2
echo "The pipeline is still running. Do not stop prematurely." >&2
echo "" >&2
echo "To gracefully pause: run /stop" >&2
echo "To check progress: run /status" >&2
echo "" >&2
echo "This stop was blocked to prevent losing orchestration state." >&2

exit 2
