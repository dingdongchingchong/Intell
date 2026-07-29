pub mod admin;
pub mod auth;
pub mod categories;
pub mod comments;
pub mod engagement;
pub mod health;
pub mod notifications;
pub mod posts;
pub mod tags;
pub mod users;

use axum::routing::get;
use axum::Router;

use crate::middleware::{CidrAllowlistLayer, RateLimitLayer};
use crate::state::AppState;
use crate::websocket;

pub fn build_router(state: AppState) -> Router<AppState> {
    let rate = RateLimitLayer::new(state.settings.rate_limit_rps, state.settings.rate_limit_burst);
    let cidr = CidrAllowlistLayer::from_cidrs(&state.settings.allowed_cidrs)
        .expect("ALLOWED_CIDRS must be valid CIDR list (e.g. 10.8.0.0/24,192.168.100.0/24)");

    let api = Router::new()
        .nest("/auth", auth::router())
        .nest("/users", users::router())
        .nest("/posts", posts::router())
        .nest("/comments", comments::router())
        .nest("/categories", categories::router())
        .nest("/tags", tags::router())
        .nest("/engagement", engagement::router())
        .nest("/notifications", notifications::router())
        .nest("/admin", admin::router())
        .route("/ws", get(websocket::ws_handler))
        .layer(rate);

    let mut router = Router::new()
        .route("/health", get(health::health))
        .route("/health/ready", get(health::ready))
        .route("/metrics", get(health::metrics))
        .nest("/api/v1", api);

    if cidr.is_enabled() {
        tracing::info!(
            cidrs = ?state.settings.allowed_cidrs,
            "source IP allowlist enabled"
        );
        router = router.layer(cidr);
    }

    router
}
