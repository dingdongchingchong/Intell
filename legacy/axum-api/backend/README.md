# CaseFlow CMS Backend (Rust)

Production-oriented CMS API built with **Axum**, **SQLx**, and **PostgreSQL**.

## Features

- Email/password auth with Argon2 + JWT access/refresh tokens
- X/Twitter OAuth 2.0 (PKCE)
- Users, posts, comments, categories, tags
- Likes, bookmarks, shares, follows
- Real-time notifications over WebSocket
- Admin dashboard, user management, post moderation
- Rate limiting, CORS, request IDs, structured logging
- OpenAPI / Swagger UI at `/swagger-ui`
- Health (`/health`, `/health/ready`) and metrics (`/metrics`)

## Quick start

```bash
# 1. Start Postgres
docker compose up -d

# 2. Configure env
cp .env.example .env

# 3. Run API
cargo run

# API:      http://localhost:8080
# Swagger:  http://localhost:8080/swagger-ui/
```

Default admin (seeded on first boot):

| Field | Value |
|-------|-------|
| Email | `admin@caseflow.local` |
| Username | `admin` |
| Password | `admin123456` |

## Project layout

```
backend/
├── migrations/              # SQLx migrations (source of truth)
├── src/db/migrations/       # Mirror copy for docs/layout requirement
├── src/
│   ├── main.rs              # Binary entrypoint
│   ├── lib.rs               # App builder
│   ├── config.rs            # Env-based settings
│   ├── error.rs             # Typed API errors
│   ├── state.rs             # Shared AppState
│   ├── models/              # Domain models
│   ├── repositories/        # SQL access
│   ├── services/            # Business logic
│   ├── middleware/          # JWT auth + rate limit
│   ├── routes/              # HTTP handlers
│   ├── websocket/           # Realtime notifications
│   └── openapi.rs           # OpenAPI document
└── tests/
```

## Auth

```bash
# Register
curl -s -X POST localhost:8080/api/v1/auth/register \
  -H 'content-type: application/json' \
  -d '{"email":"a@b.com","username":"alice","password":"secret12","display_name":"Alice"}'

# Login
curl -s -X POST localhost:8080/api/v1/auth/login \
  -H 'content-type: application/json' \
  -d '{"login":"alice","password":"secret12"}'

# Authenticated request
curl -s localhost:8080/api/v1/auth/me -H "Authorization: Bearer $ACCESS_TOKEN"
```

Twitter OAuth (optional): set `TWITTER_CLIENT_ID`, `TWITTER_CLIENT_SECRET`, `TWITTER_REDIRECT_URI`, then visit `/api/v1/auth/oauth/twitter`.

WebSocket: `ws://localhost:8080/api/v1/ws?token=<access_token>`

## Main endpoints

| Method | Path | Notes |
|--------|------|-------|
| POST | `/api/v1/auth/register` | Create account |
| POST | `/api/v1/auth/login` | JWT pair |
| POST | `/api/v1/auth/refresh` | Rotate refresh |
| GET/POST | `/api/v1/posts` | List / create |
| GET/PUT/DELETE | `/api/v1/posts/{id_or_slug}` | Post CRUD |
| GET/POST | `/api/v1/comments/posts/{post_id}` | Comments |
| POST/DELETE | `/api/v1/engagement/posts/{id}/like` | Likes |
| POST/DELETE | `/api/v1/engagement/users/{id}/follow` | Follows |
| GET | `/api/v1/notifications` | Inbox |
| GET | `/api/v1/admin/dashboard` | Admin metrics |
| PATCH | `/api/v1/admin/users/{id}` | Manage users |
| PATCH | `/api/v1/admin/posts/{id}/moderate` | Moderation |

## Roles

`admin` > `editor` > `author` > `viewer`

- **Admin**: user management, moderation, full access
- **Editor**: moderation + content
- **Author**: create/edit own posts
- **Viewer**: read + engagement

## Testing

```bash
# Unit tests (no DB)
cargo test --test auth_unit

# Integration (needs Postgres)
docker compose up -d
cargo test --test api_integration -- --ignored
```

## Production notes

- Set a long random `JWT_SECRET`
- Use managed Postgres with SSL (`?sslmode=require`)
- Put the API behind TLS termination
- Replace in-memory rate limiting with Redis for multi-instance deploys
- Configure Twitter OAuth redirect for your public URL
- Enable `RUST_LOG=info` (or JSON logging) in production

## License

MIT
