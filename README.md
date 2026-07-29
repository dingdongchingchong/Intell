# CaseFlow — Enterprise Investigation CMS

Next.js 14 frontend + Rust API (SQLx/Postgres) for legal investigation case management.

- **Vercel** hosts the Next.js app (repo root — auto-detected)
- **Fly / Railway / Render / VPS** hosts the Axum API against Neon/Supabase Postgres

> **ORM note:** Prax ORM was evaluated but is still early WIP. Production data access uses **SQLx** with typed models in `crates/caseflow-core`.

## Architecture

| Layer | Stack |
|-------|--------|
| Frontend | Next.js 14 App Router, TypeScript, Tailwind, NextAuth.js |
| API | Rust (`caseflow-core` + Axum in `crates/caseflow-api`) |
| Database | PostgreSQL (Neon / Supabase / local) |
| Desktop | Optional Tauri shell in `desktop/` (legacy SPA also in `legacy/`) |

```
cms/
├── src/                      # Next.js App Router
├── public/
├── crates/caseflow-core/     # Domain, auth, SQLx, migrations
├── crates/caseflow-api/      # Axum server + optional vercel_runtime bins
├── desktop/                  # Tauri (optional)
├── legacy/                   # Previous SPA + Axum CMS
├── package.json              # Next.js app (Vercel entry)
└── Cargo.toml                # Rust workspace
```

## Quick start

### 1. Database

```bash
cp .env.example .env
cp .env.example .env.local   # Next.js reads .env.local
# edit DATABASE_URL, JWT_SECRET, NEXTAUTH_SECRET
```

Local Postgres (Docker/Podman):

```bash
podman compose up -d   # or: docker compose up -d
# DATABASE_URL=postgres://cms:cms@127.0.0.1:5433/caseflow
```

### 2. Rust API

```bash
cargo run -p caseflow-api --bin seed     # migrate + seed admin
cargo run -p caseflow-api --bin server   # http://127.0.0.1:8080
```

Default admin: `admin` / `admin123456` (override via `SEED_ADMIN_*`).

### 3. Next.js frontend

```bash
npm install
npm run dev
```

Open http://localhost:3000 — redirects to the **Investigation Manager** (`/investigation.html`), the original CaseFlow SPA (Kanban, CSV/Excel import, localStorage). Admin API UI remains at `/login` → `/dashboard`.


## Deploy to Vercel (frontend)

Import **this repo** on [vercel.com/new](https://vercel.com/new) — leave **Root Directory** as `.` (repo root). Framework: **Next.js** (auto-detected from `package.json`).

### Environment variables

| Name | Example | Notes |
|------|---------|--------|
| `NEXTAUTH_URL` | `https://your-app.vercel.app` | Exact deployment URL |
| `NEXTAUTH_SECRET` | long random string | `openssl rand -base64 32` |
| `NEXT_PUBLIC_API_URL` | `https://api.yourdomain.com` | Public Axum API — **not** `127.0.0.1` |

Optional: `GOOGLE_CLIENT_ID` / `GOOGLE_CLIENT_SECRET`, `GITHUB_ID` / `GITHUB_SECRET`.

### After deploy

1. Host the Rust API separately (Fly.io, Railway, Render, or a VPS) with `DATABASE_URL` pointing at Neon/Supabase (use a **pooler** URL).
2. Set CORS on the API to allow your Vercel origin.
3. Redeploy the frontend if you change `NEXT_PUBLIC_API_URL`.

CLI alternative:

```bash
npx vercel          # link + preview
npx vercel --prod   # production
```

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

## Legacy

Previous Investigation Manager SPA and Axum CMS live in `legacy/spa` and `legacy/axum-api`.

## Security checklist (baseline)

- [x] JWT auth + bcrypt passwords  
- [x] Role-based permissions in Rust  
- [x] Soft delete on cases  
- [x] Audit log table + writes on key actions  
- [ ] Account lockout / email verify / attachments — scaffold next  
- [ ] Rate limiting at edge / reverse proxy  

## License

MIT
