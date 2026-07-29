use axum::extract::{Path, Query, State};
use axum::routing::{get, patch};
use axum::{Json, Router};
use uuid::Uuid;
use validator::Validate;

use crate::dto::common::{ApiResponse, PaginationQuery};
use crate::dto::users::{FollowStats, UpdateProfileRequest};
use crate::error::AppResult;
use crate::middleware::AuthUser;
use crate::models::user::PublicUser;
use crate::repositories::{EngagementRepo, UserRepo};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/me", patch(update_me))
        .route("/{id}", get(get_user))
        .route("/{id}/stats", get(follow_stats))
}

async fn update_me(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<UpdateProfileRequest>,
) -> AppResult<Json<ApiResponse<PublicUser>>> {
    body.validate()?;
    let updated = UserRepo::update_profile(
        &state.db,
        user.id,
        body.display_name.as_deref(),
        body.bio.as_deref(),
        body.avatar_url.as_deref(),
    )
    .await?;
    Ok(Json(ApiResponse::new(updated.into())))
}

async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ApiResponse<PublicUser>>> {
    let user = UserRepo::find_by_id(&state.db, id)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("user not found".into()))?;
    Ok(Json(ApiResponse::new(user.into())))
}

async fn follow_stats(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(_): Query<PaginationQuery>,
) -> AppResult<Json<ApiResponse<FollowStats>>> {
    let (followers, following) = EngagementRepo::follow_stats(&state.db, id).await?;
    Ok(Json(ApiResponse::new(FollowStats {
        user_id: id,
        followers,
        following,
    })))
}
