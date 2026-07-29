#!/usr/bin/env bash
# debug-caseflow.sh — quick health / login diagnostics
# Usage: ./deploy/secure/scripts/debug-caseflow.sh
set -euo pipefail

API_HOST="${API_HOST:-127.0.0.1}"
API_PORT="${API_PORT:-8080}"
API="http://${API_HOST}:${API_PORT}"
TS_IP="$(tailscale ip -4 2>/dev/null || true)"
PASS="${ADMIN_PASS:-admin123456}"

echo "🔍 CaseFlow Debug Tool"
echo "======================"
echo "API base: $API"

echo ""
echo "📡 Tailscale:"
if command -v tailscale >/dev/null 2>&1; then
  systemctl is-active tailscaled 2>/dev/null || true
  echo "  IP: ${TS_IP:-unknown}"
else
  echo "  ❌ tailscale not installed"
fi

echo ""
echo "🔌 Listeners :8080 / :3000"
ss -lntp 2>/dev/null | grep -E ':8080|:3000' || echo "  (none)"

echo ""
echo "🔧 Backend health ($API):"
if curl -fsS -m 3 "$API/health" ; then
  echo ""
  echo "  ✅ health OK"
else
  echo "  ❌ Backend NOT reachable at $API"
fi

if [[ -n "$TS_IP" ]]; then
  echo ""
  echo "🔧 Backend health (Tailscale $TS_IP):"
  curl -fsS -m 3 "http://${TS_IP}:8080/health" && echo "  ✅" || echo "  ❌"
fi

echo ""
echo "🔐 Login tests (password=$PASS):"
for body in \
  "{\"login\":\"admin\",\"password\":\"$PASS\"}" \
  "{\"username\":\"admin\",\"password\":\"$PASS\"}" \
  "{\"email\":\"admin@caseflow.local\",\"password\":\"$PASS\"}"
do
  echo "  POST $body"
  code=$(curl -sS -m 5 -o /tmp/cf_login.json -w '%{http_code}' \
    -X POST "$API/api/v1/auth/login" \
    -H 'Content-Type: application/json' \
    -d "$body" || echo "000")
  echo "  → HTTP $code $(head -c 120 /tmp/cf_login.json 2>/dev/null; echo)"
done

if [[ -f /tmp/cf_login.json ]] && grep -q access_token /tmp/cf_login.json 2>/dev/null; then
  TOKEN=$(python3 -c "import json;print(json.load(open('/tmp/cf_login.json'))['data']['tokens']['access_token'])" 2>/dev/null || true)
  if [[ -n "${TOKEN:-}" ]]; then
    echo ""
    echo "👤 /api/v1/auth/me"
    curl -fsS -m 5 "$API/api/v1/auth/me" -H "Authorization: Bearer $TOKEN" | head -c 200
    echo ""
  fi
fi

echo ""
echo "Done."
