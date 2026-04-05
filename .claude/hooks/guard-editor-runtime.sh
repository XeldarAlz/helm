#!/usr/bin/env bash
# ============================================================================
# guard-editor-runtime.sh — BLOCKING HOOK
# Blocks usage of UnityEditor namespace in runtime code without #if guard.
# Code using UnityEditor compiles in the Editor but fails on player build.
# This silently passes until someone tries to build, then hours of debugging.
# ============================================================================
# Trigger: PreToolUse on Edit|Write
# Exit: 2 = block, 0 = allow
# ============================================================================

set -euo pipefail

INPUT=$(cat)

FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')
NEW_CONTENT=$(echo "$INPUT" | jq -r '.tool_input.new_string // .tool_input.content // empty')

# Only check C# files
case "$FILE_PATH" in
    *.cs) ;;
    *) exit 0 ;;
esac

# Skip files already in Editor folders — editor code is fine there
case "$FILE_PATH" in
    */Editor/*|*/editor/*) exit 0 ;;
esac

# Skip if no content to check
if [ -z "$NEW_CONTENT" ]; then
    exit 0
fi

# Strip comments and strings to avoid false positives on commented-out code
STRIPPED_CONTENT=$(echo "$NEW_CONTENT" | sed 's|//.*||g; s/"[^"]*"/""/g' | sed ':a;N;$!ba;s|/\*[^*]*\*\+\([^/*][^*]*\*\+\)*/||g')

# Check if the new content uses UnityEditor namespace
if echo "$STRIPPED_CONTENT" | grep -qE '(using\s+UnityEditor|UnityEditor\.)'; then
    # Check if it's properly guarded with #if UNITY_EDITOR (check original, not stripped)
    if ! echo "$NEW_CONTENT" | grep -qE '#if\s+UNITY_EDITOR'; then
        echo "BLOCKED: UnityEditor namespace used in runtime code without #if UNITY_EDITOR guard." >&2
        echo "" >&2
        echo "  File: $FILE_PATH" >&2
        echo "" >&2
        echo "  This code compiles in the Editor but FAILS on player build." >&2
        echo "  Either:" >&2
        echo "    1. Move this file to an Editor/ folder, or" >&2
        echo "    2. Wrap the editor code with:" >&2
        echo "       #if UNITY_EDITOR" >&2
        echo "       using UnityEditor;" >&2
        echo "       #endif" >&2
        exit 2
    fi
fi

exit 0
