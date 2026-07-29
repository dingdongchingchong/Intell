# CaseFlow Desktop (Tauri)

Native desktop shell for CaseFlow CMS. It:

1. Opens the Investigation Manager UI (`frontend/`)
2. Starts the Rust API (`backend/`) on `127.0.0.1:8080`
3. Stops the API when the window closes

## Prerequisites

- Node.js 18+
- Rust (stable)
- PostgreSQL reachable via `backend/.env` (`DATABASE_URL`)
- Backend builds successfully (`cargo build` in `backend/`)
- **Linux system libraries** for Tauri (WebKitGTK / GTK)

### Fedora

```bash
sudo bash scripts/install-tauri-deps-fedora.sh
```

### Other distros

See [Tauri Linux prerequisites](https://v2.tauri.app/start/prerequisites/).

## Quick start (dev)

```bash
cd ~/projects/cms
npm install
npm run tauri:dev
```

This will:

- Build/copy the backend sidecar (`scripts/prepare-sidecar.sh`)
- Serve `frontend/` on `http://127.0.0.1:1420`
- Launch the Tauri window and spawn `caseflow_cms` against `backend/.env`

Login is unchanged (`admin` / `admin123456` unless you changed seed credentials).

## Production bundle

```bash
cd ~/projects/cms
npm install
npm run tauri:build
```

Artifacts land under `src-tauri/target/release/bundle/` (AppImage/deb on Linux, MSI on Windows, app on macOS).

The installer bundles:

- The UI (`frontend/`)
- The API binary (`binaries/caseflow-backend-*`)
- `backend/.env` and `backend/migrations` as resources

**Note:** Bundling `.env` includes DB credentials. For shared installs, replace the packaged env or point `DATABASE_URL` at a local DB and rotate secrets.

## How it fits together

| Piece | Role |
|-------|------|
| `src-tauri/` | Tauri 2 app (window + process lifecycle) |
| `frontend/` | Static SPA loaded in the webview |
| `backend/` | Axum API started as a child process |
| `scripts/prepare-sidecar.sh` | Builds API and copies it for `externalBin` |

The UI detects Tauri and always uses `http://127.0.0.1:8080` for API calls (override with `localStorage.cf_api_base` if needed).

## Troubleshooting

- **Backend did not become ready** — check Postgres, `backend/.env`, and that nothing else holds port 8080.
- **CORS errors** — desktop spawn merges Tauri origins into `CORS_ORIGINS`; also ensure `backend/.env` includes `http://tauri.localhost` / `https://tauri.localhost`.
- **Missing sidecar** — run `npm run prepare:sidecar` (or `--release`) and confirm `src-tauri/binaries/caseflow-backend-<triple>` exists.
