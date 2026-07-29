//! CaseFlow CMS — production-ready Axum + SQLx backend.

pub mod config;
pub mod db;
pub mod dto;
pub mod error;
pub mod middleware;
pub mod models;
pub mod openapi;
pub mod repositories;
pub mod routes;
pub mod services;
pub mod state;
pub mod websocket;

pub use error::{AppError, AppResult};
pub use state::AppState;

use axum::http::{header, HeaderName, HeaderValue, Method};
use axum::Router;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::config::Settings;
use crate::openapi::ApiDoc;
use crate::routes::build_router;
use crate::services::auth::AuthService;

/// Build the full HTTP application router.
pub async fn build_app(state: AppState) -> AppResult<Router> {
    let cors = build_cors(&state.settings);

    let api = build_router(state.clone())
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(cors);

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .merge(api)
        .with_state(state);

    Ok(app)
}

/// CORS that is valid with credentials (no `*` headers/methods/origins).
fn build_cors(settings: &Settings) -> CorsLayer {
    let origins: Vec<HeaderValue> = settings
        .cors_origins
        .iter()
        .filter(|o| o.as_str() != "*" && !o.is_empty())
        .filter_map(|o| o.parse().ok())
        .collect();

    // Development fallback: if no valid origins configured, be permissive (no credentials).
    if origins.is_empty() {
        return CorsLayer::permissive();
    }

    let headers = [
        header::AUTHORIZATION,
        header::CONTENT_TYPE,
        header::ACCEPT,
        header::ORIGIN,
        HeaderName::from_static("x-request-id"),
        HeaderName::from_static("x-requested-with"),
    ];

    let methods = [
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::PATCH,
        Method::DELETE,
        Method::OPTIONS,
        Method::HEAD,
    ];

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods(methods)
        .allow_headers(headers)
        .allow_credentials(true)
}

/// Seed default admin on empty database.
pub async fn seed_admin(state: &AppState) -> AppResult<()> {
    AuthService::seed_admin_if_needed(state).await
}
