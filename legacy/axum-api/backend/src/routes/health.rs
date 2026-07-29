use axum::extract::State;
use axum::Json;
use serde::Serialize;
use utoipa::ToSchema;

use crate::error::AppResult;
use crate::state::AppState;

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: String,
    pub version: &'static str,
}

#[utoipa::path(get, path = "/health", tag = "health", responses((status = 200, body = HealthResponse)))]
pub async fn health(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: state.settings.app_name.clone(),
        version: env!("CARGO_PKG_VERSION"),
    })
}

#[utoipa::path(get, path = "/health/ready", tag = "health", responses((status = 200), (status = 503)))]
pub async fn ready(State(state): State<AppState>) -> AppResult<Json<HealthResponse>> {
    sqlx::query("SELECT 1").execute(&state.db).await?;
    Ok(Json(HealthResponse {
        status: "ready",
        service: state.settings.app_name.clone(),
        version: env!("CARGO_PKG_VERSION"),
    }))
}

pub async fn metrics() -> &'static str {
    // Placeholder Prometheus text — wire metrics-exporter-prometheus in production deployments.
    "# HELP caseflow_up 1 if process is up\n# TYPE caseflow_up gauge\ncaseflow_up 1\n"
}
