use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use uuid::Uuid;

use crate::dto::common::{MessageResponse, PaginationQuery};
use crate::dto::notifications::NotificationList;
use crate::error::AppResult;
use crate::middleware::AuthUser;
use crate::services::notifications::NotificationService;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list))
        .route("/read-all", post(read_all))
        .route("/{id}/read", post(read_one))
}

async fn list(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Query(q): Query<PaginationQuery>,
) -> AppResult<Json<NotificationList>> {
    let data = NotificationService::list(&state, user.id, q.page, q.per_page).await?;
    Ok(Json(data))
}

async fn read_one(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<MessageResponse>> {
    NotificationService::mark_read(&state, user.id, id).await?;
    Ok(Json(MessageResponse {
        message: "marked read".into(),
    }))
}

async fn read_all(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> AppResult<Json<MessageResponse>> {
    let n = NotificationService::mark_all_read(&state, user.id).await?;
    Ok(Json(MessageResponse {
        message: format!("marked {n} notifications read"),
    }))
}
