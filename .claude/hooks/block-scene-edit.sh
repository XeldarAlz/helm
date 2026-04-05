#!/usr/bin/env bash
# ============================================================================
# block-scene-edit.sh — BLOCKING HOOK
# Prevents Claude from directly editing .unity, .prefab, and .asset YAML files.
# These files contain serialized references that break when text-edited.
# Use unity-mcp tools (manage_scene, manage_gameobject, manage_prefabs) instead.
# ============================================================================
# Trigger: PreToolUse on Edit|Write
# Exit: 2 = block, 0 = allow
# ============================================================================

set -euo pipefail

# Read the tool input from stdin (JSON with tool_name, file_path, etc.)
INPUT=$(cat)

# Extract the file path from the tool input
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')

if [ -z "$FILE_PATH" ]; then
    exit 0
fi

# Check if the file has a Unity binary/YAML extension
case "$FILE_PATH" in
    *.unity|*.prefab)
        echo "BLOCKED: Direct editing of scene/prefab files corrupts serialized references." >&2
        echo "" >&2
        echo "  File: $FILE_PATH" >&2
        echo "" >&2
        echo "  Instead, use unity-mcp tools:" >&2
        echo "    - manage_scene      → create/load/modify scenes" >&2
        echo "    - manage_gameobject  → create/modify GameObjects" >&2
        echo "    - manage_components  → add/configure components" >&2
        echo "    - manage_prefabs     → create/edit prefabs" >&2
        echo "    - batch_execute      → bundle multiple operations" >&2
        exit 2
        ;;
    *.asset)
        # Allow .asset files in Scripts/ or code-generated paths, block others
        case "$FILE_PATH" in
            */Scripts/*|*/Editor/*|*/Plugins/*)
                exit 0
                ;;
            *)
                echo "BLOCKED: Direct editing of .asset files can corrupt serialized data." >&2
                echo "" >&2
                echo "  File: $FILE_PATH" >&2
                echo "" >&2
                echo "  Instead, use unity-mcp tools:" >&2
                echo "    - manage_asset              → manage assets" >&2
                echo "    - manage_scriptable_object   → edit ScriptableObjects" >&2
                echo "    - manage_material            → edit materials" >&2
                exit 2
                ;;
        esac
        ;;
    *)
        exit 0
        ;;
esac
