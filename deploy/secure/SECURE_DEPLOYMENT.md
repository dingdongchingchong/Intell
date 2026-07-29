# CaseFlow CMS — VPN / Private-Network Only Deployment

**Goal:** CaseFlow must be reachable only from your company VPN, LAN, or private cloud network — never from the public internet.

Defense is layered:

1. **Network** — VPN gateway + firewall deny on `:8080` / `:5432` from `0.0.0.0/0`
2. **Bind** — App listens on a private IP or `127.0.0.1` (not `0.0.0.0` in production)
3. **App allowlist** — optional `ALLOWED_CIDRS` rejects non-VPN source IPs
4. **Auth** — JWT login still required after network access

```
Public Internet
      │
      ▼
 Firewall (deny :8080, :5432)
      │
      ▼
 VPN Gateway (OpenVPN / WireGuard / Tailscale / Cloudflare Zero Trust)
      │
      ▼
 Private net ──► CaseFlow APP (192.168.100.10:8080)
              └► PostgreSQL (127.0.0.1:5432 only)
```

---

## Choose an option

| Option | Best for | Effort |
|--------|----------|--------|
| **1. Tailscale** | Small teams, no IT staff | Lowest |
| **2. Self-hosted OpenVPN/WireGuard + UFW** | On-prem law / investigation firms | Medium |
| **3. Cloud VPC + Client VPN** | Multi-office / AWS·Azure·GCP | Higher |
| **4. Cloudflare Tunnel + Zero Trust** | Easy HTTPS + identity checks | Medium |
| **5. SSH tunnel** | Occasional admin access only | Lowest |
| **6. NGINX IP allowlist** | Existing reverse-proxy stack | Medium |

Scripts live under [`deploy/secure/`](./).

---

## Quick start (recommended: Tailscale)

```bash
# On the CaseFlow server
sudo ./deploy/secure/scripts/install-tailscale.sh

# Generate production .env bound to private / Tailscale IP
PRIVATE_IP=$(tailscale ip -4) ./deploy/secure/scripts/setup-secure-caseflow.sh
cp backend/.env.production backend/.env

# Postgres on loopback only
docker compose -f deploy/secure/docker-compose.secure.yml \
  --env-file backend/.env.production up -d
# (set DB_PASS to match DATABASE_URL)

cd backend && cargo run --release

# Firewall: only private CIDRs
sudo LAN_CIDR=192.168.100.0/24 VPN_CIDR=10.8.0.0/24 \
  ./deploy/secure/scripts/firewall-vpn-only.sh 8080

./deploy/secure/scripts/verify-not-public.sh 127.0.0.1 8080
```

Open `http://<tailscale-ip>:8080` **only while connected to Tailscale**.

Apply [`deploy/secure/tailscale/acl.json`](./tailscale/acl.json) in the Tailscale admin console.

---

## Application configuration

### Required production settings

| Variable | Secure value | Notes |
|----------|--------------|-------|
| `APP_ENV` | `production` | Enables bind safety checks |
| `APP_HOST` | Private IP or `127.0.0.1` | **Not** `0.0.0.0` unless opted in |
| `ALLOW_PUBLIC_BIND` | `false` | Opt-in only behind locked firewall |
| `ALLOWED_CIDRS` | `10.8.0.0/24,192.168.100.0/24,…` | App-level IP allowlist |
| `CORS_ORIGINS` | Private URLs only | No public origins |
| `JWT_SECRET` | ≥32 random chars | Required in production |
| `DATABASE_URL` | `…@127.0.0.1:5432…` | DB not on public interface |

Generate a production file:

```bash
PRIVATE_IP=192.168.100.10 ./deploy/secure/scripts/setup-secure-caseflow.sh
```

### What the binary enforces

- In `production`, binding `0.0.0.0` / `::` **fails** unless `ALLOW_PUBLIC_BIND=true`
- `JWT_SECRET` must be ≥ 32 characters in production
- If `ALLOWED_CIDRS` is set, requests from other IPs get **403**
- Connect info is used so peer IPs are accurate (not only `X-Forwarded-For`)

---

## Option details

### 1) Tailscale (zero-config VPN)

```bash
sudo ./deploy/secure/scripts/install-tailscale.sh
```

- Users install Tailscale and join your tailnet
- CaseFlow is never published to a public IP
- ACLs restrict who can hit `:8080`

### 2) Self-hosted OpenVPN / WireGuard + firewall

1. Install and run OpenVPN (UDP 1194) or WireGuard (UDP 51820)
2. Push a route to the CaseFlow LAN subnet
3. Run:

```bash
sudo LAN_CIDR=192.168.100.0/24 VPN_CIDR=10.8.0.0/24 \
  ./deploy/secure/scripts/firewall-vpn-only.sh 8080
```

4. Set `APP_HOST` to the LAN IP and `ALLOWED_CIDRS` to LAN+VPN pools

**UFW intent:** allow `:8080` from VPN/LAN only; `deny 8080` for everyone else; never open `:5432`.

### 3) Cloud VPC (AWS / Azure / GCP)

- Place the app in a **private subnet** (no public IP)
- Security group / NSG: inbound `8080` only from Client VPN / VPC CIDR
- Postgres: same private subnet, SG limited to app SG
- Expose VPN via AWS Client VPN, Azure VPN Gateway, or Cloudflare

(Terraform sketches can mirror the SG rules in the prompt; keep CaseFlow off public subnets.)

### 4) Cloudflare Tunnel + Zero Trust

- App binds `127.0.0.1:8080`
- `cloudflared` publishes a private hostname
- Zero Trust Access: email OTP / SSO / device posture
- No inbound firewall ports required for HTTP

### 5) SSH tunnel (admins only)

```bash
# Server .env
APP_HOST=127.0.0.1
APP_ENV=production

# User machine
ssh -N -L 8080:127.0.0.1:8080 user@caseflow-bastion
# Browse http://127.0.0.1:8080
```

### 6) NGINX IP allowlist

Use [`deploy/secure/nginx/caseflow.conf`](./nginx/caseflow.conf):

- TLS termination
- `allow` VPN/LAN CIDRs; `deny all`
- `proxy_pass` to `127.0.0.1:8080`

---

## systemd

```bash
sudo useradd -r -s /usr/sbin/nologin caseflow || true
sudo mkdir -p /opt/caseflow
sudo cp -a backend /opt/caseflow/
sudo cp deploy/secure/systemd/caseflow.service /etc/systemd/system/
# Place production .env at /opt/caseflow/backend/.env (mode 600)
cd /opt/caseflow/backend && cargo build --release
sudo systemctl daemon-reload
sudo systemctl enable --now caseflow
```

---

## Verification checklist

Run [`scripts/verify-not-public.sh`](./scripts/verify-not-public.sh) on the server, then from a **non-VPN** machine:

```bash
curl -m 5 http://<PUBLIC_IP>:8080/health   # must FAIL (timeout/refused)
curl -m 5 http://<VPN_IP>:8080/health      # must return 200 while on VPN
```

| Check | Pass criteria |
|-------|----------------|
| No public listener | `ss -lntp` shows private IP or `127.0.0.1`, or firewall drops public |
| Firewall | `:8080` and `:5432` denied from WAN |
| VPN required | Off-VPN curl fails; on-VPN succeeds |
| `APP_ENV=production` | Bind check + JWT length enforced |
| `ALLOWED_CIDRS` set | Spoofed/off-net clients get 403 |
| `CORS_ORIGINS` | Only private origins |
| Strong secrets | Unique `JWT_SECRET`, DB password, admin password |
| Logs | Access + auth failures reviewed |

---

## User onboarding

1. Issue VPN credentials (Tailscale invite / OpenVPN `.ovpn` / WireGuard config)
2. Confirm user can ping `APP_HOST`
3. Share `http://<private-ip>:8080` (or investigation SPA URL on the same host)
4. Create CaseFlow login (admin seeds first user; then create manager/viewer accounts)
5. Revoke VPN + disable CaseFlow user on departure

---

## Investigation SPA note

`investigation.html` is a browser SPA. Host it on the **same private server** (or over the VPN) — never on a public static host if case data is sensitive. Prefer:

```bash
# On the private host only
cd /opt/caseflow && python3 -m http.server 8081 --bind 192.168.100.10
```

Or serve via the same NGINX allowlisted vhost.

---

## Incident response

If you discover `:8080` open to the world:

1. Immediately `ufw deny 8080` / remove public SG rule
2. Rotate `JWT_SECRET`, DB password, and admin password
3. Review auth logs / VPN logs for unexpected sources
4. Set `APP_HOST` to private IP + `ALLOWED_CIDRS` and restart
