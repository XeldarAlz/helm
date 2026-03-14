#!/bin/bash
# Hook: Logs file creation/modification events for progress tracking
# Receives JSON on stdin with tool_input.file_path

INPUT=$(cat)
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')

if [ -z "$FILE_PATH" ]; then
    exit 0
fi

# Only track C# files and key documents
if ! echo "$FILE_PATH" | grep -qE "\.(cs|md)$"; then
    exit 0
fi

# Skip the activity log itself to prevent recursion
if echo "$FILE_PATH" | grep -qE "ACTIVITY_LOG\.md$"; then
    exit 0
fi

CWD=$(echo "$INPUT" | jq -r '.cwd // empty')
if [ -z "$CWD" ]; then
    CWD="$(pwd)"
fi

LOG_FILE="${CWD}/docs/ACTIVITY_LOG.md"
TIMESTAMP=$(date '+%Y-%m-%d %H:%M:%S')

# Only create log if docs/ directory exists
if [ ! -d "${CWD}/docs" ]; then
    exit 0
fi

# Create log file if it doesn't exist
if [ ! -f "$LOG_FILE" ]; then
    echo "# Activity Log" > "$LOG_FILE"
    echo "" >> "$LOG_FILE"
    echo "| Timestamp | Action | File |" >> "$LOG_FILE"
    echo "|-----------|--------|------|" >> "$LOG_FILE"
fi

# Determine action
TOOL_NAME=$(echo "$INPUT" | jq -r '.tool_name // "unknown"')
ACTION="$TOOL_NAME"

# Append to log
echo "| ${TIMESTAMP} | ${ACTION} | \`${FILE_PATH}\` |" >> "$LOG_FILE"

exit 0
