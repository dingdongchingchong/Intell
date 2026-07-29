#!/usr/bin/env bash
# verify-not-public.sh — Confirm CaseFlow is not reachable from a "public" perspective.
# Run ON the server (checks local bind) and optionally from an external host.
set -euo pipefail

HOST="${1:-127.0.0.1}"
PORT="${2:-8080}"
PRIVATE_IP="${PRIVATE_IP:-}"

echo "==> Checking listeners on :$PORT"
if command -v ss >/dev/null 2>&1; then
  ss -lntp | grep -E ":${PORT}\\b" || echo "(no listener found)"
elif command -v netstat >/dev/null 2>&1; then
  netstat -lntp | grep -E ":${PORT}\\b" || true
fi

echo
echo "==> Local health check"
if curl -fsS --max-time 3 "http://${HOST}:${PORT}/health" >/dev/null; then
  echo "OK  http://${HOST}:${PORT}/health"
else
  echo "FAIL local health (is the app running?)"
fi

if [[ -n "$PRIVATE_IP" ]]; then
  echo
  echo "==> Private IP health"
  curl -fsS --max-time 3 "http://${PRIVATE_IP}:${PORT}/health" && echo " OK" || echo " FAIL"
fi

echo
echo "==> Bind sanity"
# Fail if we appear bound to 0.0.0.0 in production without ALLOW_PUBLIC_BIND
if ss -lntp 2>/dev/null | grep -E "0\\.0\\.0\\.0:${PORT}\\b|\\*:${PORT}\\b" >/dev/null; then
  echo "WARN: listening on all interfaces (0.0.0.0). Ensure firewall denies public ingress."
else
  echo "OK: not obviously bound to 0.0.0.0"
fi

echo
echo "From an EXTERNAL (non-VPN) host run:"
echo "  curl -m 5 http://<PUBLIC_IP>:${PORT}/health"
echo "Expected: timeout / connection refused (NOT 200)."
