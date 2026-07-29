# Security checklist — VPN-only CaseFlow

Print or copy into your runbook. All boxes must pass before go-live.

## Network

- [ ] Server has **no** requirement to expose 8080/5432 on the public WAN
- [ ] VPN (Tailscale / OpenVPN / WireGuard / Client VPN) is required for users
- [ ] Firewall: allow 8080 only from LAN + VPN CIDRs; deny everyone else
- [ ] Firewall: PostgreSQL 5432 not reachable from WAN (prefer `127.0.0.1` bind)
- [ ] From a phone on cellular (no VPN): `curl http://PUBLIC_IP:8080/health` fails
- [ ] From VPN: health check returns 200

## Application

- [ ] `APP_ENV=production`
- [ ] `APP_HOST` is private IP or `127.0.0.1` (not `0.0.0.0` unless justified)
- [ ] `ALLOW_PUBLIC_BIND=false` (or true only with perimeter firewall + docs)
- [ ] `ALLOWED_CIDRS` lists VPN + LAN only
- [ ] `CORS_ORIGINS` lists private origins only
- [ ] `JWT_SECRET` ≥ 32 chars, unique, not in git
- [ ] Seed admin password rotated from defaults
- [ ] Swagger UI access accepted only on VPN (or disabled in prod if desired)

## Data

- [ ] Backups encrypted and stored off the app box
- [ ] `.env` mode `600`, owned by service user
- [ ] No case CSVs or dumps on public file shares

## People

- [ ] Each user has individual VPN identity
- [ ] Offboarding revokes VPN **and** CaseFlow account
- [ ] Admin MFA on VPN/IdP where available (Tailscale SSO / Cloudflare Access)

## Verify command

```bash
./deploy/secure/scripts/verify-not-public.sh 127.0.0.1 8080
```
