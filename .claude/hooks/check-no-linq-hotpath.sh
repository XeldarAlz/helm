#!/bin/bash
# Hook: Warns if LINQ is used alongside hot path methods
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

# Check if file uses System.Linq
HAS_LINQ=$(grep -c "using System.Linq" "$FILE_PATH" 2>/dev/null)
if [ "$HAS_LINQ" -eq 0 ]; then
    exit 0
fi

# Strip comments and strings before checking for hot path methods
STRIPPED=$(sed 's|//.*||g; s/"[^"]*"/""/g' "$FILE_PATH" 2>/dev/null | sed ':a;N;$!ba;s|/\*[^*]*\*\+\([^/*][^*]*\*\+\)*/||g')

# Check if file has hot path methods
HOT_METHODS=$(echo "$STRIPPED" | grep -nE "(void|IEnumerator|UniTask)\s+(Update|FixedUpdate|LateUpdate|Tick|Execute|OnUpdate|Process)\s*\(")
if [ -n "$HOT_METHODS" ]; then
    echo "WARNING: File imports System.Linq AND contains hot path methods!"
    echo "File: $FILE_PATH"
    echo ""
    echo "Hot path methods found:"
    echo "$HOT_METHODS"
    echo ""
    echo "LINQ causes heap allocations. Forbidden on hot paths."
    echo "Use for-loops, pre-allocated collections, or Span<T> instead."
    # Warning only
    exit 0
fi

exit 0
