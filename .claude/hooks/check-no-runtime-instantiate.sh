#!/bin/bash
# Hook: Warns if code uses runtime GameObject creation
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

# Skip pool/factory implementations (they're allowed to instantiate)
if echo "$FILE_PATH" | grep -qiE "(Pool|Factory|Spawner)\.cs$"; then
    exit 0
fi

# Skip test files
if echo "$FILE_PATH" | grep -qiE "(Tests?|Spec)/"; then
    exit 0
fi

ISSUES=""

# Strip comments and string literals to avoid false positives
STRIPPED=$(sed 's|//.*||g; s/"[^"]*"/""/g' "$FILE_PATH" 2>/dev/null | sed ':a;N;$!ba;s|/\*[^*]*\*\+\([^/*][^*]*\*\+\)*/||g')

# Check for Instantiate calls (not inside pool/factory context)
INSTANTIATE=$(echo "$STRIPPED" | grep -nE "\bInstantiate\s*[<(]")
if [ -n "$INSTANTIATE" ]; then
    ISSUES="${ISSUES}\nInstantiate() calls found (use object pooling instead):\n${INSTANTIATE}\n"
fi

# Check for new GameObject
NEW_GO=$(echo "$STRIPPED" | grep -nE "\bnew\s+GameObject\s*\(")
if [ -n "$NEW_GO" ]; then
    ISSUES="${ISSUES}\nnew GameObject() calls found (use pre-created prefabs from pools):\n${NEW_GO}\n"
fi

# Check for Destroy (should use pool.Return instead)
DESTROY=$(echo "$STRIPPED" | grep -nE "\bDestroy\s*\(" | grep -v "OnDestroy")
if [ -n "$DESTROY" ]; then
    ISSUES="${ISSUES}\nDestroy() calls found (use pool.Return() or SetActive(false) instead):\n${DESTROY}\n"
fi

if [ -n "$ISSUES" ]; then
    echo "WARNING: Runtime GameObject creation/destruction detected!"
    echo "File: $FILE_PATH"
    echo -e "$ISSUES"
    echo "Constraint: No runtime GameObject creation. Use object pools and prefabs."
    exit 0
fi

exit 0
