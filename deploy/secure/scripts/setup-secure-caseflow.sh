#!/usr/bin/env bash
# setup-secure-caseflow.sh — Bind CaseFlow to a private IP and write production .env
# From repo root:
#   PRIVATE_IP=100.x.y.z ./deploy/secure/scripts/setup-secure-caseflow.sh
# From backend/:
#   PRIVATE_IP=100.x.y.z ../deploy/secure/scripts/setup-secure-caseflow.sh
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
BACKEND="$ROOT/backend"
ENV_FILE="$BACKEND/.env.production"

PRIVATE_IP="${PRIVATE_IP:-$(hostname -I 2>/dev/null | awk '{print $1}')}"
APP_PORT="${APP_PORT:-8080}"
LAN_CIDR="${LAN_CIDR:-192.168.100.0/24}"
VPN_CIDR="${VPN_CIDR:-10.8.0.0/24}"
DB_PASS="${DB_PASS:-$(openssl rand -base64 24 | tr -d '/+=' | head -c 24)}"
JWT_SECRET="${JWT_SECRET:-$(openssl rand -base64 48 | tr -d '\n')}"
ADMIN_PASS="${ADMIN_PASS:-$(openssl rand -base64 12 | tr -d '/+=')}"

if [[ -z "${PRIVATE_IP}" ]]; then
  echo "Set PRIVATE_IP=192.168.x.x" >&2
  exit 1
fi

cat > "$ENV_FILE" <<EOF
# CaseFlow — VPN / private-network production
APP_NAME=caseflow_cms
APP_ENV=production
APP_HOST=${PRIVATE_IP}
APP_PORT=${APP_PORT}
APP_URL=http://${PRIVATE_IP}:${APP_PORT}
FRONTEND_URL=http://${PRIVATE_IP}:${APP_PORT}
RUST_LOG=caseflow_cms=info,tower_http=warn,sqlx=warn

DATABASE_URL=postgres://caseflow:${DB_PASS}@127.0.0.1:5432/caseflow_db
DATABASE_MAX_CONNECTIONS=20

JWT_SECRET=${JWT_SECRET}
JWT_ACCESS_TTL_SECS=900
JWT_REFRESH_TTL_SECS=604800
JWT_ISSUER=caseflow_cms

RATE_LIMIT_RPS=20
RATE_LIMIT_BURST=40

# Browser origins on the private network only (no public domains)
CORS_ORIGINS=http://${PRIVATE_IP}:${APP_PORT},http://127.0.0.1:${APP_PORT}

# Defense in depth: reject API clients outside VPN/LAN
ALLOWED_CIDRS=${LAN_CIDR},${VPN_CIDR},127.0.0.1/32,100.64.0.0/10

# Never set true unless behind a locked perimeter firewall
ALLOW_PUBLIC_BIND=false

SEED_ADMIN_EMAIL=admin@caseflow.local
SEED_ADMIN_USERNAME=admin
SEED_ADMIN_PASSWORD=${ADMIN_PASS}
SEED_ADMIN_NAME=Administrator
EOF

chmod 600 "$ENV_FILE"

echo "✅ Wrote $ENV_FILE"
echo "   APP_HOST=$PRIVATE_IP (private bind)"
echo "   Admin password: $ADMIN_PASS"
echo "   DB password:    $DB_PASS"
echo
echo "Next:"
echo "  1. Start Postgres bound to 127.0.0.1 only"
echo "  2. cp $ENV_FILE $BACKEND/.env"
echo "  3. cd $BACKEND && cargo run --release"
echo "  4. sudo ./deploy/secure/scripts/firewall-vpn-only.sh $APP_PORT"
echo "  5. Connect via VPN, open http://${PRIVATE_IP}:${APP_PORT}"
