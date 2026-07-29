#!/usr/bin/env bash
# firewall-vpn-only.sh — Block CaseFlow ports from the public internet.
# Usage: sudo ./firewall-vpn-only.sh [APP_PORT]
set -euo pipefail

APP_PORT="${1:-8080}"
# Adjust these to your LAN + VPN pools
LAN_CIDR="${LAN_CIDR:-192.168.100.0/24}"
VPN_CIDR="${VPN_CIDR:-10.8.0.0/24}"
TAILSCALE_CIDR="${TAILSCALE_CIDR:-100.64.0.0/10}"
ADMIN_SSH_CIDR="${ADMIN_SSH_CIDR:-$LAN_CIDR}"

if [[ $EUID -ne 0 ]]; then
  echo "Run as root: sudo $0" >&2
  exit 1
fi

if command -v ufw >/dev/null 2>&1; then
  echo "==> Configuring UFW (VPN/LAN only for :$APP_PORT)"
  ufw --force reset || true
  ufw default deny incoming
  ufw default allow outgoing

  # SSH only from trusted admin networks
  ufw allow from "$ADMIN_SSH_CIDR" to any port 22 proto tcp comment 'SSH from LAN/VPN'

  # VPN control plane (OpenVPN UDP / WireGuard)
  ufw allow 1194/udp comment 'OpenVPN'
  ufw allow 51820/udp comment 'WireGuard'

  # CaseFlow — private networks only (order: allow then deny)
  ufw allow from "$LAN_CIDR" to any port "$APP_PORT" proto tcp comment 'CaseFlow LAN'
  ufw allow from "$VPN_CIDR" to any port "$APP_PORT" proto tcp comment 'CaseFlow OpenVPN'
  ufw allow from "$TAILSCALE_CIDR" to any port "$APP_PORT" proto tcp comment 'CaseFlow Tailscale'
  ufw deny "$APP_PORT"/tcp comment 'Block CaseFlow from public internet'

  # Postgres should never be public
  ufw deny 5432/tcp comment 'Block PostgreSQL publicly'

  ufw --force enable
  ufw status verbose
  echo "✅ UFW ready — port $APP_PORT only from LAN/VPN"
  exit 0
fi

if command -v firewall-cmd >/dev/null 2>&1; then
  echo "==> Configuring firewalld"
  firewall-cmd --permanent --new-zone=caseflow-private || true
  firewall-cmd --permanent --zone=caseflow-private --add-source="$LAN_CIDR"
  firewall-cmd --permanent --zone=caseflow-private --add-source="$VPN_CIDR"
  firewall-cmd --permanent --zone=caseflow-private --add-source="$TAILSCALE_CIDR"
  firewall-cmd --permanent --zone=caseflow-private --add-port="${APP_PORT}/tcp"
  firewall-cmd --permanent --zone=public --remove-port="${APP_PORT}/tcp" || true
  firewall-cmd --permanent --zone=public --add-port=1194/udp || true
  firewall-cmd --permanent --zone=public --add-port=51820/udp || true
  firewall-cmd --reload
  firewall-cmd --list-all-zones | sed -n '/caseflow-private/,/^$/p'
  echo "✅ firewalld ready"
  exit 0
fi

echo "No ufw/firewalld found — install one or apply nftables/iptables manually." >&2
exit 1
