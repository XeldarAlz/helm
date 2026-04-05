#!/usr/bin/env bash
set -euo pipefail

# Helm Pipeline Installer
# Copies the Claude Code pipeline config (.claude/) into a target directory.
#
# Usage:
#   ./install-pipeline.sh /path/to/unity/project
#   curl -fsSL https://raw.githubusercontent.com/XeldarAlz/helm/main/scripts/install-pipeline.sh | bash -s /path/to/unity/project

REPO_URL="https://github.com/XeldarAlz/helm.git"
BRANCH="main"

# --- Colors ---
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

info()  { echo -e "${BLUE}[helm]${NC} $1"; }
ok()    { echo -e "${GREEN}[helm]${NC} $1"; }
warn()  { echo -e "${YELLOW}[helm]${NC} $1"; }
fail()  { echo -e "${RED}[helm]${NC} $1" >&2; exit 1; }

# --- Parse arguments ---
TARGET_DIR="${1:-}"

if [ -z "$TARGET_DIR" ]; then
    fail "Usage: install-pipeline.sh /path/to/unity/project"
fi

TARGET_DIR="$(cd "$TARGET_DIR" 2>/dev/null && pwd)" || fail "Directory does not exist: $1"

# --- Check prerequisites ---
info "Checking prerequisites..."

if ! command -v git &>/dev/null; then
    fail "git is not installed. Install it first."
fi

if ! command -v claude &>/dev/null; then
    warn "Claude Code CLI not found. Install it: https://docs.anthropic.com/en/docs/claude-code"
    warn "The pipeline will be installed but won't run without Claude Code."
fi

# --- Check if .claude/ already exists ---
if [ -d "$TARGET_DIR/.claude" ]; then
    warn ".claude/ already exists in $TARGET_DIR"
    read -p "Overwrite? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        info "Aborted."
        exit 0
    fi
    rm -rf "$TARGET_DIR/.claude"
fi

# --- Clone and copy ---
TEMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TEMP_DIR"' EXIT

info "Downloading Helm pipeline..."
git clone --depth 1 --branch "$BRANCH" "$REPO_URL" "$TEMP_DIR/helm" 2>/dev/null

info "Installing pipeline to $TARGET_DIR/.claude/"
cp -r "$TEMP_DIR/helm/.claude" "$TARGET_DIR/.claude"

# --- Verify ---
SKILL_COUNT=$(find "$TARGET_DIR/.claude/skills" -name "SKILL.md" 2>/dev/null | wc -l | tr -d ' ')
HOOK_COUNT=$(find "$TARGET_DIR/.claude/hooks" -name "*.sh" 2>/dev/null | wc -l | tr -d ' ')
COMMAND_COUNT=$(find "$TARGET_DIR/.claude/commands" -name "*.md" 2>/dev/null | wc -l | tr -d ' ')

ok "Pipeline installed successfully!"
echo ""
echo "  Skills:   $SKILL_COUNT"
echo "  Hooks:    $HOOK_COUNT"
echo "  Commands: $COMMAND_COUNT"
echo ""
info "Quick start:"
echo ""
echo "  cd $TARGET_DIR"
echo "  claude"
echo "  > /build-game"
echo ""
info "Or run stages individually:"
echo ""
echo "  > /game-idea       # Describe your game"
echo "  > /architect       # Generate architecture"
echo "  > /plan-workflow   # Create execution plan"
echo "  > /orchestrate     # Let agents build it"
echo ""
