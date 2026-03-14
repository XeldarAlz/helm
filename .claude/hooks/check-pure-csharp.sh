#!/bin/bash
# Hook: Validates that C# files in Logic/Core/Systems directories don't import UnityEngine
# Receives JSON on stdin with tool_input.file_path

INPUT=$(cat)
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')

# Exit if no file path or not a CS file
if [ -z "$FILE_PATH" ]; then
    exit 0
fi

# Only check files in logic directories (not adapters/views/MonoBehaviours)
if echo "$FILE_PATH" | grep -qiE "(Logic|Core|Systems|Domain|Models|Services)/.*\.cs$"; then
    # Skip if it's explicitly an adapter, view, or MonoBehaviour
    if echo "$FILE_PATH" | grep -qiE "(Adapter|View|Mono|Behaviour|Inspector|Editor|Drawer)/"; then
        exit 0
    fi

    # Check for UnityEngine imports
    if [ -f "$FILE_PATH" ]; then
        UNITY_IMPORTS=$(grep -n "using UnityEngine" "$FILE_PATH" 2>/dev/null)
        if [ -n "$UNITY_IMPORTS" ]; then
            echo "BLOCKED: Pure C# logic file contains UnityEngine imports!"
            echo "File: $FILE_PATH"
            echo ""
            echo "Violations:"
            echo "$UNITY_IMPORTS"
            echo ""
            echo "Pure C# logic files must NOT depend on UnityEngine."
            echo "Move Unity-specific code to an adapter/view MonoBehaviour."
            exit 2
        fi
    fi
fi

exit 0
