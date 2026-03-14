#!/bin/bash
# Hook: Basic C# syntax verification
# Receives JSON on stdin with tool_input.file_path

INPUT=$(cat)
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')

if [ -z "$FILE_PATH" ]; then
    exit 0
fi

if ! echo "$FILE_PATH" | grep -qE "\.cs$"; then
    exit 0
fi

if [ ! -f "$FILE_PATH" ]; then
    exit 0
fi

ISSUES=""

# Check for unmatched braces
OPEN_BRACES=$(tr -cd '{' < "$FILE_PATH" | wc -c)
CLOSE_BRACES=$(tr -cd '}' < "$FILE_PATH" | wc -c)
if [ "$OPEN_BRACES" -ne "$CLOSE_BRACES" ]; then
    ISSUES="${ISSUES}\nERROR: Unmatched braces — ${OPEN_BRACES} opening vs ${CLOSE_BRACES} closing\n"
fi

# Check for namespace declaration
HAS_NAMESPACE=$(grep -cE "^namespace\s+" "$FILE_PATH" 2>/dev/null)
if [ "$HAS_NAMESPACE" -eq 0 ]; then
    ISSUES="${ISSUES}\nWARNING: No namespace declaration found\n"
fi

# Check for type declaration
HAS_TYPE=$(grep -cE "(class|struct|interface|enum|record)\s+[A-Z]" "$FILE_PATH" 2>/dev/null)
if [ "$HAS_TYPE" -eq 0 ]; then
    ISSUES="${ISSUES}\nWARNING: No type declaration found (class/struct/interface/enum/record)\n"
fi

if [ -n "$ISSUES" ]; then
    echo "Syntax check issues in: $FILE_PATH"
    echo -e "$ISSUES"
    exit 0
fi

exit 0
