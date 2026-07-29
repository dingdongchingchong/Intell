use axum::extract::{Path, Query, State};
use axum::routing::{delete, get};
use axum::{Json, Router};
use uuid::Uuid;

use crate::dto::common::{ApiResponse, MessageResponse, Paginated, PaginationQuery};
use crate::dto::posts::CreateCommentRequest;
use crate::error::AppResult;
use crate::middleware::AuthUser;
use crate::models::comment::Comment;
use crate::repositories::CommentRepo;
use crate::services::content::ContentService;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/posts/{post_id}", get(list).post(create))
        .route("/{id}", delete(soft_delete))
}

async fn list(
    State(state): State<AppState>,
    Path(post_id): Path<Uuid>,
    Query(q): Query<PaginationQuery>,
) -> AppResult<Json<ApiResponse<Paginated<Comment>>>> {
    let (items, total) =
        CommentRepo::list_for_post(&state.db, post_id, q.limit(), q.offset()).await?;
    Ok(Json(ApiResponse::new(Paginated {
        items,
        page: q.page.max(1),
        per_page: q.per_page.clamp(1, 100),
        total,
    })))
}

async fn create(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(post_id): Path<Uuid>,
    Json(body): Json<CreateCommentRequest>,
) -> AppResult<Json<ApiResponse<Comment>>> {
    let comment =
        ContentService::add_comment(&state, &user, post_id, &body.body, body.parent_id).await?;
    Ok(Json(ApiResponse::new(comment)))
}

async fn soft_delete(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<MessageResponse>> {
    let comment = CommentRepo::find_by_id(&state.db, id)
        .await?
        .ok_or_else(|| crate::error::AppError::NotFound("comment not found".into()))?;
    if comment.author_id != user.id && !user.role.can_moderate() {
        return Err(crate::error::AppError::Forbidden(
            "cannot delete this comment".into(),
        ));
    }
    CommentRepo::soft_delete(&state.db, id).await?;
    Ok(Json(MessageResponse {
        message: "comment deleted".into(),
    }))
}
