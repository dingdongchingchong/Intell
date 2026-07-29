use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use uuid::Uuid;

use crate::dto::common::{ApiResponse, MessageResponse, PaginationQuery};
use crate::dto::posts::ShareRequest;
use crate::error::AppResult;
use crate::middleware::AuthUser;
use crate::models::engagement::Bookmark;
use crate::repositories::EngagementRepo;
use crate::services::engagement::EngagementService;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/posts/{post_id}/like", post(like).delete(unlike))
        .route("/posts/{post_id}/bookmark", post(bookmark).delete(unbookmark))
        .route("/posts/{post_id}/share", post(share))
        .route("/users/{user_id}/follow", post(follow).delete(unfollow))
        .route("/bookmarks", get(my_bookmarks))
}

async fn like(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(post_id): Path<Uuid>,
) -> AppResult<Json<MessageResponse>> {
    EngagementService::like_post(&state, &user, post_id).await?;
    Ok(Json(MessageResponse {
        message: "liked".into(),
    }))
}

async fn unlike(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(post_id): Path<Uuid>,
) -> AppResult<Json<MessageResponse>> {
    EngagementService::unlike_post(&state, user.id, post_id).await?;
    Ok(Json(MessageResponse {
        message: "unliked".into(),
    }))
}

async fn bookmark(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(post_id): Path<Uuid>,
) -> AppResult<Json<MessageResponse>> {
    EngagementService::bookmark(&state, &user, post_id).await?;
    Ok(Json(MessageResponse {
        message: "bookmarked".into(),
    }))
}

async fn unbookmark(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(post_id): Path<Uuid>,
) -> AppResult<Json<MessageResponse>> {
    EngagementService::unbookmark(&state, user.id, post_id).await?;
    Ok(Json(MessageResponse {
        message: "bookmark removed".into(),
    }))
}

async fn share(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(post_id): Path<Uuid>,
    Json(body): Json<ShareRequest>,
) -> AppResult<Json<MessageResponse>> {
    let platform = body.platform.as_deref().unwrap_or("internal");
    EngagementService::share(&state, &user, post_id, platform).await?;
    Ok(Json(MessageResponse {
        message: "shared".into(),
    }))
}

async fn follow(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(user_id): Path<Uuid>,
) -> AppResult<Json<MessageResponse>> {
    EngagementService::follow(&state, &user, user_id).await?;
    Ok(Json(MessageResponse {
        message: "following".into(),
    }))
}

async fn unfollow(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(user_id): Path<Uuid>,
) -> AppResult<Json<MessageResponse>> {
    EngagementService::unfollow(&state, user.id, user_id).await?;
    Ok(Json(MessageResponse {
        message: "unfollowed".into(),
    }))
}

async fn my_bookmarks(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Query(q): Query<PaginationQuery>,
) -> AppResult<Json<ApiResponse<Vec<Bookmark>>>> {
    let items =
        EngagementRepo::list_bookmarks(&state.db, user.id, q.limit(), q.offset()).await?;
    Ok(Json(ApiResponse::new(items)))
}
