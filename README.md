# CaseFlow — Enterprise Investigation CMS

Next.js 14 frontend + Rust API (SQLx/Postgres) for legal investigation case management. Deploy frontend on Vercel; run the API locally or as a long-lived service (Neon/Supabase Postgres).

> **ORM note:** Prax ORM was evaluated but is still early WIP. Production data access uses **SQLx** with typed models in `crates/caseflow-core`.

## Architecture

| Layer | Stack |
|-------|--------|
| Frontend | Next.js 14 App Router, TypeScript, Tailwind, NextAuth.js |
| API | Rust (`caseflow-core` + Axum server / Vercel `vercel_runtime` bins) |
| Database | PostgreSQL (Neon / Supabase / local) |
| Desktop | Optional Tauri shell in `desktop/` (legacy SPA also in `legacy/`) |

```
cms/
├── frontend/                 # Next.js app
├── crates/caseflow-core/     # Domain, auth, SQLx, migrations
├── api/                      # Axum server + Vercel function bins
├── desktop/                  # Tauri (optional)
├── legacy/                   # Previous SPA + Axum CMS
├── Cargo.toml                # Rust workspace
└── vercel.json
```

## Quick start

### 1. Database

Create a Postgres database and set `DATABASE_URL` in `.env` (copy from `.env.example`).

```bash
cp .env.example .env
# edit DATABASE_URL, JWT_SECRET, NEXTAUTH_SECRET
```

### 2. Rust API

```bash
# from repo root
cargo run -p caseflow-api --bin seed     # migrate + seed admin
cargo run -p caseflow-api --bin server   # http://127.0.0.1:8080
```

Default admin: `admin` / `admin123456` (override via `SEED_ADMIN_*`).

### 3. Next.js frontend

```bash
cd frontend
cp ../.env.example .env.local   # or symlink env vars
# ensure NEXT_PUBLIC_API_URL=http://127.0.0.1:8080
# ensure NEXTAUTH_SECRET and NEXTAUTH_URL are set
npm install
npm run dev
```

Open http://localhost:3000 → login → Dashboard / Cases / Kanban / Users.

## API surface

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/v1/health` | Health |
| POST | `/api/v1/auth/login` | Login → JWT |
| GET | `/api/v1/dashboard` | Stats |
| GET/POST | `/api/v1/cases` | List / create |
| GET/PUT/DELETE | `/api/v1/cases/:id` | Read / update / soft-delete |
| PATCH | `/api/v1/cases/:id/stage` | Stage change |
| GET | `/api/v1/cases/clients` | Client names |
| GET | `/api/v1/cases/next-id` | Next `YY-XXXX` |
| GET/POST | `/api/v1/users` | Users (RBAC) |

Roles: `admin`, `manager`, `investigator`, `viewer`.

## Vercel deployment

1. Push repo; import on Vercel with root directory = repo (or set `frontend` as root and point `NEXT_PUBLIC_API_URL` at your hosted API).
2. Set env: `NEXTAUTH_SECRET`, `NEXTAUTH_URL`, `NEXT_PUBLIC_API_URL`, OAuth keys as needed.
3. Host the Rust API separately (Fly.io, Railway, Render, or your VPS) against Neon/Supabase — serverless cold starts + Postgres connection pooling need a pooler (Neon pooler / PgBouncer).

Vercel Rust function bins live under `api/src/bin/{auth,cases,users,dashboard,health}.rs` for gradual migration to `vercel_runtime`; the Axum `server` binary is the recommended production API today.

## Legacy

Previous Investigation Manager SPA and Axum CMS live in `legacy/spa` and `legacy/axum-api`. Start scripts under `scripts/` may still target those paths.

## Security checklist (baseline)

- [x] JWT auth + bcrypt passwords  
- [x] Role-based permissions in Rust  
- [x] Soft delete on cases  
- [x] Audit log table + writes on key actions  
- [ ] Account lockout / email verify / attachments — scaffold next  
- [ ] Rate limiting at edge / reverse proxy  

## License

MIT
