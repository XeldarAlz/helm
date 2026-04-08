#!/bin/bash
# Hook: Validates C# naming conventions
# Receives JSON on stdin with tool_input.file_path
#
# Conventions:
#   Private/Protected fields: _lowerCamelCase
#   Public fields:            lowerCamelCase
#   Properties:               UpperCamelCase
#   Static readonly:          UpperCamelCase
#   Constants:                UPPER_SNAKE_CASE

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

# Check for classes/structs/interfaces not in PascalCase
NON_PASCAL_TYPES=$(echo "$STRIPPED" | grep -nE "^\s*(public|internal|private|protected)?\s*(sealed\s+|abstract\s+|static\s+|partial\s+)*(class|struct|interface|enum|record)\s+[a-z]")
if [ -n "$NON_PASCAL_TYPES" ]; then
    ISSUES="${ISSUES}\nWARNING — Types must use PascalCase:\n${NON_PASCAL_TYPES}\n"
fi

# Check for private/protected fields not using _lowerCamelCase (should start with _)
# Exclude: const, static readonly, event, delegate, properties (have { })
PRIVATE_FIELDS_BAD=$(echo "$STRIPPED" | grep -nE "^\s*(private|protected)\s+(?!const |static readonly |event |delegate )(\w+\s+)+[^_][a-zA-Z]" | grep -v "[{}()]" | grep -v "void\|class\|struct\|interface\|enum\|sealed\|abstract\|override\|virtual\|async")
if [ -n "$PRIVATE_FIELDS_BAD" ]; then
    ISSUES="${ISSUES}\nWARNING — Private/protected fields should use _lowerCamelCase:\n${PRIVATE_FIELDS_BAD}\n"
fi

# Check for const fields not using UPPER_SNAKE_CASE
CONST_BAD=$(echo "$STRIPPED" | grep -nE "^\s*(public|private|protected|internal)?\s*(static\s+)?const\s+\w+\s+[a-z]" | grep -v "UPPER")
if [ -n "$CONST_BAD" ]; then
    ISSUES="${ISSUES}\nWARNING — Constant fields should use UPPER_SNAKE_CASE:\n${CONST_BAD}\n"
fi

if [ -n "$ISSUES" ]; then
    echo "Naming convention issues in: $FILE_PATH"
    echo -e "$ISSUES"
    echo "(Warning only — review and fix if needed)"
    # Exit 0 = warning, doesn't block
    exit 0
fi

exit 0
