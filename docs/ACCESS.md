# CaseFlow CMS — Remote Access Guide

## Overview

CaseFlow can be reached without a public domain using:

1. **Tailscale** (preferred) — mesh VPN to your node / MagicDNS hostname
2. **SSH tunnel** — forward local `3000` + `8080` to the server
3. **LAN/VPN** — only if firewall + `ALLOWED_CIDRS` allow it

Default CMS login (seed): `admin` / `admin123456` (change in production).

---

## Option A: Tailscale

1. Install Tailscale and join the same tailnet.
2. Open **http://\<TAILSCALE_IP_OR_MAGICDNS\>:3000**  
   Set `SERVER_HOST` / `MAGIC_DNS` when generating access scripts (see below).
3. Sign in.

Backend API: `http://\<TAILSCALE_IP_OR_MAGICDNS\>:8080` (health: `/health`).

---

## Option B: SSH tunnel

### Server requirements

- `sshd` listening (port 22)
- Frontend on `127.0.0.1:3000` (or `0.0.0.0:3000`)
- Backend on `127.0.0.1:8080` (or `0.0.0.0:8080`)
- User’s **public** SSH key in the account that runs the tunnel target (see SSH key management)

### Client steps

```bash
# From the CaseFlow repo (or a copy of tunnel.sh shared by admin)
chmod +x tunnel.sh
SERVER_USER=youruser SERVER_HOST=100.x.y.z ./tunnel.sh
```

Then open **http://localhost:3000**. Keep the tunnel terminal open.

Override host/user:

```bash
SERVER_USER=youruser SERVER_HOST=100.x.y.z ./tunnel.sh
```

Manual equivalent:

```bash
ssh -N \
  -L 3000:127.0.0.1:3000 \
  -L 8080:127.0.0.1:8080 \
  youruser@100.x.y.z
```

The Investigation Manager resolves the API to `http://127.0.0.1:8080` when the page is on localhost, which matches this tunnel.

### First-time SSH key

If the user has no key, `tunnel.sh` generates `~/.ssh/id_ed25519` and prints the **public** key. An admin adds it in the CMS (**Users → SSH Access**) or manually:

```bash
# On the server (as the SSH login user)
mkdir -p ~/.ssh && chmod 700 ~/.ssh
echo 'ssh-ed25519 AAAA... comment' >> ~/.ssh/authorized_keys
chmod 600 ~/.ssh/authorized_keys
```

---

## SSH key management (admin API)

Admin JWT required. Keys are stored in a dedicated file (default under the CMS deploy tree) and can be synced into OpenSSH `authorized_keys`.

| Method | Path |
|--------|------|
| List | `GET /api/v1/admin/ssh-keys` |
| Add | `POST /api/v1/admin/ssh-keys` `{"username":"alice","public_key":"ssh-ed25519 …"}` |
| Remove | `DELETE /api/v1/admin/ssh-keys/{username}` |

Env:

```bash
# Absolute path to the managed keys file (one key per line, "# username" comment)
SSH_AUTHORIZED_KEYS_PATH=/path/to/cms/deploy/ssh/caseflow_authorized_keys
```

Sync into live SSH access (server):

```bash
./deploy/ssh/sync-authorized-keys.sh
```

---

## Generate per-user instructions

```bash
SERVER_HOST=100.x.y.z MAGIC_DNS=caseflow.tailnet-xxxx.ts.net \
  ./scripts/generate-access-script.sh user@example.com alice
# → /tmp/caseflow-access-alice.txt
```

---

## Run services (server)

```bash
# Postgres
docker start caseflow-postgres   # or: podman start caseflow-postgres

# API
cd ~/projects/cms/backend && cargo run

# UI
cd ~/projects/cms/frontend && python3 -m http.server 3000 --bind 0.0.0.0
```

Health check:

```bash
curl -sS http://127.0.0.1:8080/health
curl -sS -o /dev/null -w '%{http_code}\n' http://127.0.0.1:3000/
```

---

## Troubleshooting

| Symptom | Fix |
|---------|-----|
| `Address already in use` on 3000/8080 | Something already serves that port — reuse it or `fuser -k 3000/tcp` |
| Tunnel OK but blank page | Confirm frontend is bound and listening on the server |
| Login works, API fails on localhost | Confirm second forward for **8080**; CORS includes `http://localhost:3000` |
| SSH `Permission denied (publickey)` | Public key not in authorized keys; re-add via CMS + sync |
| CIDR blocked | Client IP must be in `ALLOWED_CIDRS` (Tailscale `100.64.0.0/10` or loopback for tunnel origin) |

Note: With an SSH tunnel, the HTTP client IP seen by the backend is usually **127.0.0.1** (local to the server), so loopback must remain in `ALLOWED_CIDRS`.
