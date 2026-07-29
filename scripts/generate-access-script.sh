#!/usr/bin/env bash
# Generate a shareable access instructions file for a CMS user.
# Usage: ./scripts/generate-access-script.sh <email> <cms-username>

set -euo pipefail

USER_EMAIL="${1:-}"
CMS_USERNAME="${2:-}"
SERVER_HOST="${SERVER_HOST:-}"
MAGIC_DNS="${MAGIC_DNS:-}"
OUT_DIR="${OUT_DIR:-/tmp}"

if [[ -z "$USER_EMAIL" || -z "$CMS_USERNAME" ]]; then
  echo "Usage: SERVER_HOST=100.x.y.z MAGIC_DNS=name.tailnet.ts.net $0 <user-email> <cms-username>"
  exit 1
fi

if [[ -z "$SERVER_HOST" || -z "$MAGIC_DNS" ]]; then
  echo "Set SERVER_HOST and MAGIC_DNS (Tailscale IP / MagicDNS hostname)." >&2
  exit 1
fi

OUT="${OUT_DIR}/caseflow-access-${CMS_USERNAME}.txt"

cat > "$OUT" <<EOF
CaseFlow CMS — Access Instructions
==================================

Hello,

You have been granted access to CaseFlow CMS.

OPTION 1 — Tailscale (preferred)
--------------------------------
1. Install Tailscale: https://tailscale.com/download
2. Join the team network (admin invite)
3. Open: http://${SERVER_HOST}:3000
   or:   http://${MAGIC_DNS}:3000
4. Sign in with your CMS username

OPTION 2 — SSH tunnel (no Tailscale app on phone / restricted networks)
----------------------------------------------------------------------
1. Ask your admin for tunnel.sh (or download from the CMS Access page)
2. chmod +x tunnel.sh && ./tunnel.sh
3. If you have no SSH key, the script generates one — send the public
   key to your admin so they can add it under Users → SSH Access
4. When the tunnel is up, open: http://localhost:3000
5. Sign in with your CMS username

Login
-----
Username: ${CMS_USERNAME}
Password: (sent separately / set when you accept your invite)
Email:    ${USER_EMAIL}

Security
--------
• Prefer Tailscale or SSH tunnels — do not expose ports publicly
• Never share passwords or private keys
• Contact admin@caseflow.local if something looks wrong

Generated: $(date -Iseconds)
EOF

echo "Wrote $OUT"
cat "$OUT"
