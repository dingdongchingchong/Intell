#!/usr/bin/env bash
# Merge CaseFlow-managed SSH keys into ~/.ssh/authorized_keys (idempotent).
# Usage: ./deploy/ssh/sync-authorized-keys.sh

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
SRC="${SSH_AUTHORIZED_KEYS_PATH:-$ROOT/deploy/ssh/caseflow_authorized_keys}"
DEST="${HOME}/.ssh/authorized_keys"
MARKER_BEGIN="# BEGIN caseflow_cms"
MARKER_END="# END caseflow_cms"

mkdir -p "${HOME}/.ssh"
chmod 700 "${HOME}/.ssh"
touch "$DEST"
chmod 600 "$DEST"

if [[ ! -f "$SRC" ]]; then
  echo "No managed keys file at $SRC (nothing to sync)."
  exit 0
fi

TMP="$(mktemp)"
# Strip previous managed block
awk -v b="$MARKER_BEGIN" -v e="$MARKER_END" '
  $0==b {skip=1; next}
  $0==e {skip=0; next}
  !skip {print}
' "$DEST" > "$TMP"

{
  cat "$TMP"
  echo "$MARKER_BEGIN"
  # Only lines managed by CaseFlow
  grep -E ' # caseflow:' "$SRC" || true
  echo "$MARKER_END"
} > "$DEST"

rm -f "$TMP"
chmod 600 "$DEST"
echo "Synced CaseFlow keys → $DEST"
grep -c 'caseflow:' "$DEST" || true
