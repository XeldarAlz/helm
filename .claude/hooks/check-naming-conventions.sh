#!/bin/bash
# Hook: Validates C# naming conventions
# Receives JSON on stdin with tool_input.file_path

INPUT=$(cat)
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')

if [ -z "$FILE_PATH" ]; then
    exit 0
fi

# Only check C# files
if ! echo "$FILE_PATH" | grep -qE "\.cs$"; then
    exit 0
fi

if [ ! -f "$FILE_PATH" ]; then
    exit 0
fi

ISSUES=""

# Strip comments and string literals to avoid false positives
STRIPPED=$(sed 's|//.*||g; s/"[^"]*"/""/g' "$FILE_PATH" 2>/dev/null | sed ':a;N;$!ba;s|/\*[^*]*\*\+\([^/*][^*]*\*\+\)*/||g')

# Check for public fields that should be properties (lowercase start = likely a field not property)
PUBLIC_FIELDS=$(echo "$STRIPPED" | grep -nE "^\s*public\s+(int|float|string|bool|double|long|byte|char|decimal|short|uint|ulong|ushort|sbyte|var)\s+[a-z]" | grep -v "[()]")
if [ -n "$PUBLIC_FIELDS" ]; then
    ISSUES="${ISSUES}\nWARNING — Public fields should be properties with PascalCase:\n${PUBLIC_FIELDS}\n"
fi

# Check for classes/structs/interfaces not in PascalCase
NON_PASCAL_TYPES=$(echo "$STRIPPED" | grep -nE "^\s*(public|internal|private|protected)?\s*(sealed\s+|abstract\s+|static\s+|partial\s+)*(class|struct|interface|enum|record)\s+[a-z]")
if [ -n "$NON_PASCAL_TYPES" ]; then
    ISSUES="${ISSUES}\nWARNING — Types must use PascalCase:\n${NON_PASCAL_TYPES}\n"
fi

if [ -n "$ISSUES" ]; then
    echo "Naming convention issues in: $FILE_PATH"
    echo -e "$ISSUES"
    echo "(Warning only — review and fix if needed)"
    # Exit 0 = warning, doesn't block
    exit 0
fi

exit 0
