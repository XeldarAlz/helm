#!/bin/bash
# Hook: Warns about unused private members and unused imports in C# files
# Receives JSON on stdin with tool_input.file_path
# Exit 0 = warning only, does not block

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

# Skip test files and editor scripts
if echo "$FILE_PATH" | grep -qE "(Tests?/|\.Tests?\.|\.Test$|/Editor/)"; then
    exit 0
fi

# Find the project root (where Assets/ lives) by walking up
PROJECT_ROOT=""
DIR=$(dirname "$FILE_PATH")
while [ "$DIR" != "/" ]; do
    if [ -d "$DIR/Assets" ]; then
        PROJECT_ROOT="$DIR"
        break
    fi
    DIR=$(dirname "$DIR")
done

# Fallback: search from the file's own directory
if [ -z "$PROJECT_ROOT" ]; then
    PROJECT_ROOT=$(dirname "$FILE_PATH")
fi

SEARCH_DIR="$PROJECT_ROOT/Assets"
if [ ! -d "$SEARCH_DIR" ]; then
    SEARCH_DIR="$PROJECT_ROOT"
fi

ISSUES=""

# --- Check unused using statements ---
while IFS= read -r line; do
    # Extract namespace from "using Foo.Bar;"
    ns=$(echo "$line" | sed 's/^.*using\s\+//' | sed 's/\s*;\s*$//' | sed 's/^\s*//')
    # Skip common always-needed namespaces
    case "$ns" in
        System|UnityEngine|VContainer|MessagePipe) continue ;;
    esac
    # Get the short name (last segment) to check if any type from it is used
    short=$(echo "$ns" | awk -F'.' '{print $NF}')
    # Count references to the short name outside the using line itself
    refs=$(grep -c "$short" "$FILE_PATH" 2>/dev/null)
    if [ "$refs" -le 1 ]; then
        ISSUES="${ISSUES}\n  - Possibly unused import: using $ns"
    fi
done < <(grep -E "^\s*using\s+[A-Z]" "$FILE_PATH" | grep -v "^#" | grep -v "static\s")

# --- Check unused private methods ---
while IFS= read -r line; do
    method=$(echo "$line" | grep -oE '\b[A-Z][a-zA-Z0-9]+\s*\(' | head -1 | sed 's/\s*($//')
    if [ -z "$method" ]; then
        continue
    fi
    # Skip Unity callbacks
    case "$method" in
        Awake|Start|Update|FixedUpdate|LateUpdate|OnEnable|OnDisable|OnDestroy|OnValidate|OnDrawGizmos|OnDrawGizmosSelected|OnGUI|OnApplicationPause|OnApplicationFocus|OnApplicationQuit|Reset|OnBecameVisible|OnBecameInvisible|OnCollisionEnter|OnCollisionExit|OnCollisionStay|OnCollisionEnter2D|OnCollisionExit2D|OnCollisionStay2D|OnTriggerEnter|OnTriggerExit|OnTriggerStay|OnTriggerEnter2D|OnTriggerExit2D|OnTriggerStay2D|OnPointerClick|OnPointerDown|OnPointerUp|OnPointerEnter|OnPointerExit|OnDrag|OnBeginDrag|OnEndDrag|OnDrop|OnScroll|OnSelect|OnDeselect|OnSubmit|OnCancel|OnMove|OnBeforeSerialize|OnAfterDeserialize|Construct|Dispose|Configure)
            continue ;;
    esac
    # Count references to this method in the same file (excluding declaration line)
    refs=$(grep -c "\b${method}\b" "$FILE_PATH" 2>/dev/null)
    if [ "$refs" -le 1 ]; then
        ISSUES="${ISSUES}\n  - Possibly unused private method: $method()"
    fi
done < <(grep -E "^\s*private\s+.*\s+[A-Z][a-zA-Z0-9]+\s*\(" "$FILE_PATH" | grep -v "//")

# --- Check unused private fields ---
while IFS= read -r line; do
    field=$(echo "$line" | grep -oE '_[a-z][a-zA-Z0-9]*' | head -1)
    if [ -z "$field" ]; then
        continue
    fi
    # Count references in the file (excluding the declaration)
    refs=$(grep -c "\b${field}\b" "$FILE_PATH" 2>/dev/null)
    if [ "$refs" -le 1 ]; then
        ISSUES="${ISSUES}\n  - Possibly unused private field: $field"
    fi
done < <(grep -E "^\s*(\[.*\]\s*)*(private|protected)\s+.*\s+_[a-z]" "$FILE_PATH" | grep -v "//" | grep -v "SerializeField")

if [ -n "$ISSUES" ]; then
    echo "Possible unused code in: $FILE_PATH"
    echo -e "$ISSUES"
    echo ""
    echo "Verify and remove unused members before review. (Warning only)"
    exit 0
fi

exit 0
