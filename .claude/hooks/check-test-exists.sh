#!/bin/bash
# Hook: Warns if a logic C# file has no corresponding test file
# Receives JSON on stdin with tool_input.file_path

INPUT=$(cat)
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')

if [ -z "$FILE_PATH" ]; then
    exit 0
fi

# Only check C# files in logic directories
if ! echo "$FILE_PATH" | grep -qiE "(Logic|Core|Systems|Domain|Services)/.*\.cs$"; then
    exit 0
fi

# Skip interfaces
FILENAME=$(basename "$FILE_PATH")
if echo "$FILENAME" | grep -qE "^I[A-Z].*\.cs$"; then
    exit 0
fi

# Skip test files themselves
if echo "$FILE_PATH" | grep -qiE "Tests?/"; then
    exit 0
fi

# Derive expected test file name
CLASS_NAME="${FILENAME%.cs}"
TEST_NAME="${CLASS_NAME}Tests.cs"

# Get project working directory from hook input
CWD=$(echo "$INPUT" | jq -r '.cwd // empty')
if [ -z "$CWD" ]; then
    CWD="$(pwd)"
fi

# Search for the test file
FOUND_TEST=$(find "$CWD" -name "$TEST_NAME" -type f 2>/dev/null | head -1)

if [ -z "$FOUND_TEST" ]; then
    echo "REMINDER: No test file found for ${CLASS_NAME}"
    echo "Logic file: $FILE_PATH"
    echo "Expected test: ${TEST_NAME}"
    echo ""
    echo "Every logic class must have corresponding unit tests."
    # Warning only
    exit 0
fi

exit 0
