#!/usr/bin/env bash
set -euo pipefail

echo "==> Building frontend..."
npm run build

echo "==> Building Tauri app (DMG)..."
npx tauri build --bundles dmg

DMG_DIR="src-tauri/target/release/bundle/dmg"
if [ -d "$DMG_DIR" ]; then
  echo ""
  echo "==> Build complete! DMG located at:"
  ls -1 "$DMG_DIR"/*.dmg 2>/dev/null || echo "  (no DMG found in $DMG_DIR)"
else
  echo "==> Warning: DMG directory not found at $DMG_DIR"
  exit 1
fi
