#!/usr/bin/env bash
# install-tailscale.sh — Fastest path to VPN-only CaseFlow access for small teams.
# Run from repo root:
#   sudo ./deploy/secure/scripts/install-tailscale.sh
set -euo pipefail

if [[ $EUID -ne 0 ]]; then
  echo "Run as root from the repo root, e.g.:" >&2
  echo "  cd ~/projects/cms && sudo ./deploy/secure/scripts/install-tailscale.sh" >&2
  exit 1
fi

if ! command -v tailscale >/dev/null 2>&1; then
  if command -v dnf >/dev/null 2>&1; then
    echo "==> Installing Tailscale via dnf"
    dnf install -y tailscale
  else
    echo "==> Installing Tailscale via official install script"
    curl -fsSL https://tailscale.com/install.sh | sh
  fi
else
  echo "==> Tailscale already installed: $(command -v tailscale)"
fi

echo "==> Starting tailscaled"
systemctl enable --now tailscaled

echo "==> Bringing Tailscale up (browser login may open)"
tailscale up --hostname="${HOSTNAME_OVERRIDE:-caseflow}"

IP="$(tailscale ip -4)"
echo
echo "✅ Tailscale up"
echo "   Access CaseFlow at: http://${IP}:8080"
echo "   Only devices on your Tailnet can reach this host."
echo
echo "Next (from repo root ~/projects/cms):"
echo "  PRIVATE_IP=${IP} ./deploy/secure/scripts/setup-secure-caseflow.sh"
echo "  cp backend/.env.production backend/.env   # backs up? copy carefully"
echo "  sudo ./deploy/secure/scripts/firewall-vpn-only.sh 8080"
echo "  cd backend && cargo run --release"
