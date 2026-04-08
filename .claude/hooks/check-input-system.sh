#!/bin/bash
# Hook: Validates Input System usage patterns
# Catches: legacy Input API, missing Enable/Disable, input in FixedUpdate
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

# Skip test files
if echo "$FILE_PATH" | grep -qiE "(Tests?|Spec)/"; then
    exit 0
fi

# Skip Editor-only files
if echo "$FILE_PATH" | grep -qE "/Editor/"; then
    exit 0
fi

ISSUES=""

# Strip comments and string literals to avoid false positives
STRIPPED=$(sed 's|//.*||g; s/"[^"]*"/""/g' "$FILE_PATH" 2>/dev/null | sed ':a;N;$!ba;s|/\*[^*]*\*\+\([^/*][^*]*\*\+\)*/||g')

# --- Check 1: Legacy Input API usage ---
LEGACY_INPUT=$(echo "$STRIPPED" | grep -nE "\bInput\.(GetKey|GetAxis|GetButton|GetMouseButton|mousePosition|touches|GetTouch|touchCount|anyKey|inputString)\b")
if [ -n "$LEGACY_INPUT" ]; then
    ISSUES="${ISSUES}\nBLOCKING — Legacy Input API detected (use New Input System):\n${LEGACY_INPUT}\n"
    ISSUES="${ISSUES}Fix: Use generated PlayerControls class with callbacks or polling via ReadValue/WasPressedThisFrame.\n"
fi

# --- Check 2: Missing Enable/Disable for input controls ---
# If file references PlayerControls, InputActionAsset, or InputAction, it must have Enable() in OnEnable and Disable() in OnDisable
HAS_INPUT_REF=$(echo "$STRIPPED" | grep -cE "(PlayerControls|InputActionAsset|InputAction\b|_controls\.|_inputActions\.)" 2>/dev/null)
if [ "$HAS_INPUT_REF" -gt 0 ]; then
    # Check for Enable() call in/near OnEnable
    HAS_ENABLE=$(echo "$STRIPPED" | grep -cE "\.(Enable|Player\.Enable|UI\.Enable)\(\)" 2>/dev/null)
    # Check for Disable() call in/near OnDisable
    HAS_DISABLE=$(echo "$STRIPPED" | grep -cE "\.(Disable|Player\.Disable|UI\.Disable)\(\)" 2>/dev/null)

    if [ "$HAS_ENABLE" -eq 0 ]; then
        ISSUES="${ISSUES}\nWARNING — Input controls referenced but no Enable() call found!\n"
        ISSUES="${ISSUES}Input actions MUST be enabled in OnEnable() to receive input.\n"
        ISSUES="${ISSUES}Example: _controls.Player.Enable(); in OnEnable()\n"
    fi

    if [ "$HAS_DISABLE" -eq 0 ]; then
        ISSUES="${ISSUES}\nWARNING — Input controls referenced but no Disable() call found!\n"
        ISSUES="${ISSUES}Input actions MUST be disabled in OnDisable() to prevent leaks.\n"
        ISSUES="${ISSUES}Example: _controls.Player.Disable(); in OnDisable()\n"
    fi

    # Check that callbacks are unsubscribed (if subscribed with +=, must have -=)
    SUBSCRIBES=$(echo "$STRIPPED" | grep -oE "\+= On[A-Z]\w+" | sed 's/+= //' | sort -u)
    if [ -n "$SUBSCRIBES" ]; then
        for CALLBACK in $SUBSCRIBES; do
            UNSUB_COUNT=$(echo "$STRIPPED" | grep -cE "-= ${CALLBACK}\b" 2>/dev/null)
            if [ "$UNSUB_COUNT" -eq 0 ]; then
                ISSUES="${ISSUES}\nWARNING — Input callback '${CALLBACK}' subscribed (+= ) but never unsubscribed (-=)!\n"
                ISSUES="${ISSUES}Unsubscribe in OnDisable() to prevent memory leaks and ghost callbacks.\n"
            fi
        done
    fi
fi

# --- Check 3: Input reading inside FixedUpdate ---
# Extract FixedUpdate body (rough heuristic: lines between FixedUpdate and next method)
IN_FIXED_UPDATE=false
INPUT_IN_FIXED=""
while IFS= read -r line; do
    if echo "$line" | grep -qE "void\s+FixedUpdate\s*\("; then
        IN_FIXED_UPDATE=true
        continue
    fi
    if [ "$IN_FIXED_UPDATE" = true ]; then
        # Detect next method declaration = end of FixedUpdate
        if echo "$line" | grep -qE "^\s*(private|public|protected|internal|void|static|async|override|virtual)\s+\w+.*\("; then
            IN_FIXED_UPDATE=false
            continue
        fi
        # Check for input reads inside FixedUpdate
        INPUT_READ=$(echo "$line" | grep -E "(ReadValue|WasPressedThisFrame|WasReleasedThisFrame|IsPressed|_moveInput|_controls\.\w+\.\w+\.Read)")
        if [ -n "$INPUT_READ" ]; then
            INPUT_IN_FIXED="${INPUT_IN_FIXED}\n${INPUT_READ}"
        fi
    fi
done <<< "$STRIPPED"

if [ -n "$INPUT_IN_FIXED" ]; then
    ISSUES="${ISSUES}\nWARNING — Input reading detected inside FixedUpdate!\n"
    ISSUES="${ISSUES}${INPUT_IN_FIXED}\n"
    ISSUES="${ISSUES}Read input in Update() and cache the values. Apply physics forces in FixedUpdate using cached values.\n"
fi

# --- Check 4: InputActionAsset field without [SerializeField] ---
BARE_ASSET=$(echo "$STRIPPED" | grep -nE "^\s*(private|protected)\s+InputActionAsset\s+\w+\s*;" | grep -v "SerializeField")
if [ -n "$BARE_ASSET" ]; then
    ISSUES="${ISSUES}\nWARNING — InputActionAsset field without [SerializeField]:\n${BARE_ASSET}\n"
    ISSUES="${ISSUES}The asset reference must be serialized to be assigned in the Inspector.\n"
fi

if [ -n "$ISSUES" ]; then
    # Check if any issue is BLOCKING
    if echo "$ISSUES" | grep -q "BLOCKING"; then
        echo "INPUT SYSTEM ERROR in: $FILE_PATH"
        echo -e "$ISSUES"
        exit 2
    else
        echo "Input System issues in: $FILE_PATH"
        echo -e "$ISSUES"
        exit 0
    fi
fi

exit 0
