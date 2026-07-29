use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;
use validator::Validate;

use crate::dto::common::{ApiResponse, MessageResponse, Paginated};
use crate::dto::posts::{CreatePostRequest, PostDetail, UpdatePostRequest};
use crate::error::AppResult;
use crate::middleware::{AuthUser, OptionalAuthUser};
use crate::models::post::Post;
use crate::services::content::ContentService;
use crate::state::AppState;

#[derive(Debug, Deserialize, IntoParams)]
pub struct PostListQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per")]
    pub per_page: u32,
    pub category_id: Option<Uuid>,
    pub q: Option<String>,
}

fn default_page() -> u32 { 1 }
fn default_per() -> u32 { 20 }

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_posts).post(create_post))
        .route("/{id_or_slug}", get(get_post).put(update_post).delete(delete_post))
}

async fn list_posts(
    State(state): State<AppState>,
    Query(q): Query<PostListQuery>,
) -> AppResult<Json<ApiResponse<Paginated<Post>>>> {
    let data = ContentService::list_published(
        &state,
        q.page,
        q.per_page,
        q.category_id,
        q.q.as_deref(),
    )
    .await?;
    Ok(Json(ApiResponse::new(data)))
}

async fn create_post(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<CreatePostRequest>,
) -> AppResult<Json<ApiResponse<Post>>> {
    body.validate()?;
    let post = ContentService::create_post(&state, &user, body).await?;
    Ok(Json(ApiResponse::new(post)))
}

async fn get_post(
    State(state): State<AppState>,
    Path(id_or_slug): Path<String>,
    OptionalAuthUser(viewer): OptionalAuthUser,
) -> AppResult<Json<ApiResponse<PostDetail>>> {
    let detail =
        ContentService::get_post(&state, &id_or_slug, viewer.map(|u| u.id)).await?;
    Ok(Json(ApiResponse::new(detail)))
}

async fn update_post(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id_or_slug): Path<String>,
    Json(body): Json<UpdatePostRequest>,
) -> AppResult<Json<ApiResponse<Post>>> {
    body.validate()?;
    let id = resolve_post_id(&state, &id_or_slug).await?;
    let post = ContentService::update_post(&state, &user, id, body).await?;
    Ok(Json(ApiResponse::new(post)))
}

async fn delete_post(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id_or_slug): Path<String>,
) -> AppResult<Json<MessageResponse>> {
    let id = resolve_post_id(&state, &id_or_slug).await?;
    ContentService::delete_post(&state, &user, id).await?;
    Ok(Json(MessageResponse {
        message: "post deleted".into(),
    }))
}

async fn resolve_post_id(state: &AppState, id_or_slug: &str) -> AppResult<Uuid> {
    if let Ok(id) = Uuid::parse_str(id_or_slug) {
        return Ok(id);
    }
    crate::repositories::PostRepo::find_by_slug(&state.db, id_or_slug)
        .await?
        .map(|p| p.id)
        .ok_or_else(|| crate::error::AppError::NotFound("post not found".into()))
}
