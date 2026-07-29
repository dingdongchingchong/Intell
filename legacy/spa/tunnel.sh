#!/usr/bin/env bash
# CaseFlow CMS — SSH tunnel to frontend (:3000) and API (:8080)
# Usage: ./tunnel.sh
# Optional overrides: SERVER_USER=… SERVER_HOST=… ./tunnel.sh

set -euo pipefail

SERVER_USER="${SERVER_USER:-huntersthompson}"
SERVER_HOST="${SERVER_HOST:-100.68.147.74}"
FRONTEND_PORT="${FRONTEND_PORT:-3000}"
BACKEND_PORT="${BACKEND_PORT:-8080}"
SSH_OPTS="${SSH_OPTS:--o ServerAliveInterval=30 -o ServerAliveCountMax=3 -o ExitOnForwardFailure=yes}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}CaseFlow CMS SSH Tunnel${NC}"
echo "====================================="
echo "Server: ${SERVER_USER}@${SERVER_HOST}"
echo ""

pick_pubkey() {
  if [[ -f "${HOME}/.ssh/id_ed25519.pub" ]]; then
    echo "${HOME}/.ssh/id_ed25519.pub"
  elif [[ -f "${HOME}/.ssh/id_rsa.pub" ]]; then
    echo "${HOME}/.ssh/id_rsa.pub"
  else
    echo ""
  fi
}

PUB="$(pick_pubkey)"
if [[ -z "$PUB" ]]; then
  echo -e "${YELLOW}No SSH key found. Generating ed25519 key…${NC}"
  mkdir -p "${HOME}/.ssh"
  chmod 700 "${HOME}/.ssh"
  ssh-keygen -t ed25519 -f "${HOME}/.ssh/id_ed25519" -N "" -C "caseflow-$(whoami)@$(hostname -s 2>/dev/null || echo host)"
  PUB="${HOME}/.ssh/id_ed25519.pub"
  echo -e "${GREEN}Key generated.${NC}"
  echo ""
  echo -e "${YELLOW}Send this public key to your CaseFlow admin (Users → SSH Access):${NC}"
  echo "-------------------------------------"
  cat "$PUB"
  echo "-------------------------------------"
  echo ""
  echo "After your key is added, run this script again."
  exit 1
fi

echo -e "${YELLOW}Testing SSH…${NC}"
if ! ssh -o ConnectTimeout=8 -o BatchMode=yes -q ${SSH_OPTS} "${SERVER_USER}@${SERVER_HOST}" exit; then
  echo -e "${RED}Cannot connect to ${SERVER_USER}@${SERVER_HOST}${NC}"
  echo ""
  echo "Check:"
  echo "  1. Server is online and sshd is running"
  echo "  2. Your public key was added by an admin"
  echo "  3. Firewall / Tailscale / network path allows SSH"
  echo ""
  echo "Your public key:"
  echo "-------------------------------------"
  cat "$PUB"
  echo "-------------------------------------"
  exit 1
fi

echo -e "${GREEN}SSH OK — opening tunnels${NC}"
echo "  Frontend → http://localhost:${FRONTEND_PORT}"
echo "  Backend  → http://localhost:${BACKEND_PORT}"
echo ""
echo -e "${YELLOW}Keep this terminal open. Ctrl+C to disconnect.${NC}"
echo ""

# Prefer unused local ports if defaults are busy
ensure_local_port() {
  local port="$1"
  if ss -lnt 2>/dev/null | grep -q ":${port} " || netstat -lnt 2>/dev/null | grep -q ":${port} "; then
    echo -e "${YELLOW}Warning: localhost:${port} already in use — tunnel may fail.${NC}" >&2
  fi
}
ensure_local_port "$FRONTEND_PORT"
ensure_local_port "$BACKEND_PORT"

exec ssh ${SSH_OPTS} \
  -L "${FRONTEND_PORT}:127.0.0.1:${FRONTEND_PORT}" \
  -L "${BACKEND_PORT}:127.0.0.1:${BACKEND_PORT}" \
  -N \
  "${SERVER_USER}@${SERVER_HOST}"
