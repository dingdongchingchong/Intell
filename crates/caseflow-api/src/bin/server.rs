//! Local development API server (Axum) sharing caseflow-core with Vercel functions.
//! Run: `cargo run -p caseflow-api --bin server`

use std::net::SocketAddr;

use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderMap, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, patch, post};
use axum::{Json, Router};
use caseflow_api::{err_value, require_user, user_id};
use caseflow_core::auth::{authorize, Permission};
use caseflow_core::config::Settings;
use caseflow_core::db;
use caseflow_core::error::AppError;
use caseflow_core::models::{
    CaseListQuery, CreateCaseRequest, CreateUserRequest, LoginRequest, UpdateCaseRequest,
    UpdateStageRequest,
};
use caseflow_core::services::{auth as auth_svc, cases as cases_svc, users as users_svc};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    settings: Settings,
}

struct ApiError(AppError);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status =
            StatusCode::from_u16(self.0.status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status, Json(err_value(&self.0))).into_response()
    }
}

impl From<AppError> for ApiError {
    fn from(value: AppError) -> Self {
        Self(value)
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // Binary target is `server`, lib is `caseflow_api` / `caseflow_core`
                "server=debug,caseflow_api=debug,caseflow_core=debug,tower_http=info,sqlx=warn".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Ignore unknown CLI flags like --verbose from `cargo run -- --verbose`
    let _args: Vec<String> = std::env::args().skip(1).collect();

    eprintln!("[caseflow] loading settings…");
    let settings = Settings::from_env().map_err(|e| {
        eprintln!("[caseflow] config error: {e}");
        e
    })?;

    eprintln!("[caseflow] connecting to database…");
    let pool = db::create_pool(&settings).await.map_err(|e| {
        eprintln!(
            "[caseflow] database connection failed: {e}\n\
             Hint: start Postgres with `podman start caseflow_cms_db` or `docker compose up -d`\n\
             Expected DATABASE_URL like postgres://cms:cms@127.0.0.1:5433/caseflow"
        );
        e
    })?;

    eprintln!("[caseflow] running migrations…");
    db::migrate(&pool).await?;
    if let Some(id) = auth_svc::seed_admin(&pool, &settings).await? {
        tracing::info!(%id, "seeded admin user");
        eprintln!("[caseflow] seeded admin user {id}");
    }

    let state = AppState { pool, settings };
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([header::AUTHORIZATION, header::CONTENT_TYPE, header::ACCEPT]);

    let app = Router::new()
        .route(
            "/api/v1/health",
            get(|| async { Json(json!({"status":"ok"})) }),
        )
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/dashboard", get(dashboard))
        .route("/api/v1/cases", get(list_cases).post(create_case))
        .route("/api/v1/cases/clients", get(list_clients))
        .route("/api/v1/cases/next-id", get(next_case_id))
        .route(
            "/api/v1/cases/{id}",
            get(get_case).put(update_case).delete(delete_case),
        )
        .route("/api/v1/cases/{id}/stage", patch(update_stage))
        .route("/api/v1/users", get(list_users).post(create_user))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port: u16 = std::env::var("API_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(8080);
    // Local default: loopback. Production/Docker/ngrok: set API_BIND=0.0.0.0
    let bind_host = std::env::var("API_BIND").unwrap_or_else(|_| "127.0.0.1".into());
    let addr: SocketAddr = format!("{bind_host}:{port}")
        .parse()
        .map_err(|e| anyhow::anyhow!("invalid API_BIND/API_PORT: {e}"))?;
    tracing::info!(%addr, "CaseFlow API listening");
    eprintln!("[caseflow] listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<Value>, ApiError> {
    let res = auth_svc::login(&state.pool, &state.settings, body).await?;
    Ok(Json(json!(res)))
}

async fn dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    let claims = require_user(&headers, &state.settings)?;
    authorize(&claims.role, Permission::CaseRead)?;
    let stats = cases_svc::dashboard_stats(&state.pool).await?;
    Ok(Json(json!({ "stats": stats })))
}

async fn list_cases(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<CaseListQuery>,
) -> Result<Json<Value>, ApiError> {
    let claims = require_user(&headers, &state.settings)?;
    authorize(&claims.role, Permission::CaseRead)?;
    let (cases, total) = cases_svc::list_cases(&state.pool, q).await?;
    Ok(Json(json!({ "cases": cases, "total": total })))
}

async fn create_case(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateCaseRequest>,
) -> Result<Json<Value>, ApiError> {
    let claims = require_user(&headers, &state.settings)?;
    authorize(&claims.role, Permission::CaseCreate)?;
    let actor = user_id(&claims)?;
    let case = cases_svc::create_case(&state.pool, actor, body).await?;
    Ok(Json(json!({ "case": case })))
}

async fn get_case(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, ApiError> {
    let claims = require_user(&headers, &state.settings)?;
    authorize(&claims.role, Permission::CaseRead)?;
    let case = cases_svc::get_case(&state.pool, id).await?;
    Ok(Json(json!({ "case": case })))
}

async fn update_case(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateCaseRequest>,
) -> Result<Json<Value>, ApiError> {
    let claims = require_user(&headers, &state.settings)?;
    authorize(&claims.role, Permission::CaseUpdate)?;
    let actor = user_id(&claims)?;
    let case = cases_svc::update_case(&state.pool, actor, id, body).await?;
    Ok(Json(json!({ "case": case })))
}

async fn update_stage(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateStageRequest>,
) -> Result<Json<Value>, ApiError> {
    let claims = require_user(&headers, &state.settings)?;
    authorize(&claims.role, Permission::CaseUpdate)?;
    let actor = user_id(&claims)?;
    let case = cases_svc::update_stage(&state.pool, actor, id, body).await?;
    Ok(Json(json!({ "case": case })))
}

async fn delete_case(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, ApiError> {
    let claims = require_user(&headers, &state.settings)?;
    authorize(&claims.role, Permission::CaseDelete)?;
    let actor = user_id(&claims)?;
    cases_svc::soft_delete(&state.pool, actor, id).await?;
    Ok(Json(json!({ "ok": true })))
}

async fn list_clients(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    let claims = require_user(&headers, &state.settings)?;
    authorize(&claims.role, Permission::CaseRead)?;
    let clients = cases_svc::list_clients(&state.pool).await?;
    Ok(Json(json!({ "clients": clients })))
}

async fn next_case_id(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    let claims = require_user(&headers, &state.settings)?;
    authorize(&claims.role, Permission::CaseCreate)?;
    let case_number = cases_svc::next_case_number(&state.pool).await?;
    Ok(Json(json!({ "case_number": case_number })))
}

async fn list_users(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    let claims = require_user(&headers, &state.settings)?;
    authorize(&claims.role, Permission::UserRead)?;
    let users = users_svc::list_users(&state.pool).await?;
    Ok(Json(json!({ "users": users })))
}

async fn create_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateUserRequest>,
) -> Result<Json<Value>, ApiError> {
    let claims = require_user(&headers, &state.settings)?;
    authorize(&claims.role, Permission::UserCreate)?;
    let actor = user_id(&claims)?;
    let user = users_svc::create_user(&state.pool, actor, body).await?;
    Ok(Json(json!({ "user": user })))
}
