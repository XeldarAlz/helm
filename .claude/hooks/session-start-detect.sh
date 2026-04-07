#!/bin/bash
# session-start-detect.sh — SessionStart hook
# Checks for interrupted orchestration and injects context about it.
# Non-blocking (always exit 0). Outputs JSON to stdout for context injection.

INPUT=$(cat)
CWD=$(echo "$INPUT" | jq -r '.cwd // empty' 2>/dev/null)
if [ -z "$CWD" ]; then
    CWD="$(pwd)"
fi

PROGRESS="$CWD/docs/PROGRESS.md"

# No progress file = no interrupted work
if [ ! -f "$PROGRESS" ]; then
    exit 0
fi

# Extract status (macOS-compatible sed)
STATUS=$(sed -n 's/^## Status: //p' "$PROGRESS" | head -1 | tr -d '[:space:]')

# Only care about running or paused orchestrations
if [ "$STATUS" != "running" ] && [ "$STATUS" != "paused" ]; then
    exit 0
fi

# Extract phase info for context
PHASE=$(sed -n 's/^## Phase: //p' "$PROGRESS" | head -1)
PHASE_NAME=$(sed -n 's/^## Phase Name: //p' "$PROGRESS" | head -1)

# Inject context via stdout JSON
echo "{\"additionalContext\": \"INTERRUPTED ORCHESTRATION DETECTED: Phase ${PHASE:-unknown}, ${PHASE_NAME:-unknown phase}, status: ${STATUS}. Run /continue to resume or /status for details.\"}"

exit 0
