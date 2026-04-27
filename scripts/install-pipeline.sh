#!/usr/bin/env bash
set -euo pipefail

# Helm Pipeline Installer
# Copies the Claude Code pipeline config (.claude/) into a target directory.
#
# Default path: downloads the latest published pipeline tarball (no git, no GUI).
# Fallback: sparse git clone of `.claude/` from main if the release is unavailable.
#
# Usage:
#   ./install-pipeline.sh /path/to/unity/project
#   ./install-pipeline.sh --tag v0.1.0 /path/to/unity/project
#   ./install-pipeline.sh --from-source /path/to/unity/project
#   curl -fsSL https://raw.githubusercontent.com/XeldarAlz/helm/main/scripts/install-pipeline.sh | bash -s /path/to/unity/project

REPO_SLUG="XeldarAlz/helm"
REPO_URL="https://github.com/${REPO_SLUG}.git"
RELEASE_BASE="https://github.com/${REPO_SLUG}/releases/download"
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

usage() {
    cat <<EOF
Usage: install-pipeline.sh [options] /path/to/unity/project

Options:
  --tag <version>    Install a specific pipeline release (e.g. v0.1.0).
  --from-source      Skip release tarball; sparse-clone .claude/ from main.
  -h, --help         Show this help.

Default: fetches the latest pipeline release tarball, falls back to sparse-checkout.
EOF
}

# --- Parse arguments ---
TARGET_DIR=""
PINNED_VERSION=""
FROM_SOURCE=0

while [ $# -gt 0 ]; do
    case "$1" in
        --tag)
            [ $# -ge 2 ] || fail "--tag requires a version argument"
            PINNED_VERSION="${2#v}"
            shift 2
            ;;
        --tag=*)
            PINNED_VERSION="${1#--tag=}"
            PINNED_VERSION="${PINNED_VERSION#v}"
            shift
            ;;
        --from-source)
            FROM_SOURCE=1
            shift
            ;;
        -h|--help)
            usage; exit 0
            ;;
        --)
            shift
            ;;
        -*)
            fail "Unknown option: $1 (try --help)"
            ;;
        *)
            if [ -z "$TARGET_DIR" ]; then
                TARGET_DIR="$1"
            else
                fail "Unexpected argument: $1"
            fi
            shift
            ;;
    esac
done

if [ -z "$TARGET_DIR" ]; then
    usage >&2
    exit 1
fi

TARGET_DIR="$(cd "$TARGET_DIR" 2>/dev/null && pwd)" || fail "Directory does not exist: $1"

# --- Prerequisites (curl is always required; git only for source fallback) ---
info "Checking prerequisites..."
command -v curl &>/dev/null || fail "curl is not installed. Install it first."
command -v tar  &>/dev/null || fail "tar is not installed. Install it first."

if ! command -v claude &>/dev/null; then
    warn "Claude Code CLI not found. Install it: https://docs.anthropic.com/en/docs/claude-code"
    warn "The pipeline will be installed but won't run without Claude Code."
fi

# --- Confirm overwrite if .claude/ already exists ---
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

# --- Working dir ---
TEMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TEMP_DIR"' EXIT

# --- Install paths ---
verify_sha256() {
    local archive="$1" digest_file="$2"
    if command -v shasum &>/dev/null; then
        ( cd "$(dirname "$archive")" && shasum -a 256 -c "$(basename "$digest_file")" >/dev/null 2>&1 )
    elif command -v sha256sum &>/dev/null; then
        ( cd "$(dirname "$archive")" && sha256sum -c "$(basename "$digest_file")" >/dev/null 2>&1 )
    else
        warn "No sha256 tool available; skipping checksum verification."
        return 0
    fi
}

install_from_tarball() {
    local url="$1" sha_url="$2"
    # Save under the original basename so the published .sha256 (which lists
    # the archive's original filename) verifies cleanly via `shasum -c`.
    local archive_name; archive_name="$(basename "$url")"
    local archive_path="$TEMP_DIR/$archive_name"
    info "Fetching $url"
    if ! curl -fsSL --retry 2 -o "$archive_path" "$url"; then
        return 1
    fi

    if curl -fsSL --retry 2 -o "${archive_path}.sha256" "$sha_url" 2>/dev/null; then
        if ! verify_sha256 "$archive_path" "${archive_path}.sha256"; then
            fail "Checksum verification failed for $url"
        fi
        info "Checksum verified."
    else
        warn "No checksum file at $sha_url — skipping verification."
    fi

    mkdir -p "$TEMP_DIR/extract"
    tar -xzf "$archive_path" -C "$TEMP_DIR/extract"
    [ -d "$TEMP_DIR/extract/.claude" ] || fail "Tarball does not contain .claude/"
    cp -r "$TEMP_DIR/extract/.claude" "$TARGET_DIR/.claude"
    return 0
}

install_from_source() {
    command -v git &>/dev/null || fail "git is not installed. Install git or use a published --tag."
    info "Sparse-cloning $REPO_URL (.claude/ only)..."
    git clone --depth 1 --filter=blob:none --sparse --branch "$BRANCH" "$REPO_URL" "$TEMP_DIR/helm" >/dev/null 2>&1
    git -C "$TEMP_DIR/helm" sparse-checkout set .claude >/dev/null
    [ -d "$TEMP_DIR/helm/.claude" ] || fail "Sparse checkout did not produce .claude/"
    cp -r "$TEMP_DIR/helm/.claude" "$TARGET_DIR/.claude"
}

# --- Resolve install strategy ---
if [ -n "$PINNED_VERSION" ]; then
    TAG="pipeline-v${PINNED_VERSION}"
    TARBALL_URL="${RELEASE_BASE}/${TAG}/helm-pipeline-${PINNED_VERSION}.tar.gz"
    SHA_URL="${TARBALL_URL}.sha256"
else
    TARBALL_URL="${RELEASE_BASE}/pipeline-latest/helm-pipeline.tar.gz"
    SHA_URL="${TARBALL_URL}.sha256"
fi

if [ "$FROM_SOURCE" -eq 1 ]; then
    install_from_source
elif install_from_tarball "$TARBALL_URL" "$SHA_URL"; then
    :
else
    if [ -n "$PINNED_VERSION" ]; then
        fail "Could not download pinned release $TAG from $TARBALL_URL"
    fi
    warn "Release tarball unavailable; falling back to git sparse-checkout."
    install_from_source
fi

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
