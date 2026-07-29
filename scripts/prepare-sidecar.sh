#!/usr/bin/env bash
# Build the CaseFlow backend and copy it into src-tauri/binaries for Tauri externalBin.
# Usage:
#   ./scripts/prepare-sidecar.sh           # debug build
#   ./scripts/prepare-sidecar.sh --release # release build

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BACKEND_DIR="$ROOT/backend"
BIN_DIR="$ROOT/src-tauri/binaries"
TRIPLE="$(rustc -vV | sed -n 's/^host: //p')"

PROFILE="debug"
CARGO_ARGS=()
if [[ "${1:-}" == "--release" ]]; then
  PROFILE="release"
  CARGO_ARGS+=(--release)
fi

mkdir -p "$BIN_DIR"

echo "Building caseflow_cms ($PROFILE)…"
(
  cd "$BACKEND_DIR"
  CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$BACKEND_DIR/target}" cargo build "${CARGO_ARGS[@]}"
)

SRC="$BACKEND_DIR/target/$PROFILE/caseflow_cms"
if [[ ! -f "$SRC" ]]; then
  echo "error: missing binary at $SRC" >&2
  exit 1
fi

DEST="$BIN_DIR/caseflow-backend-${TRIPLE}"
cp -f "$SRC" "$DEST"
chmod +x "$DEST"
echo "Sidecar ready: $DEST"
