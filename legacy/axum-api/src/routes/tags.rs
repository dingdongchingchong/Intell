use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use slug::slugify;
use validator::Validate;

use crate::dto::common::ApiResponse;
use crate::dto::posts::CreateTagRequest;
use crate::error::AppResult;
use crate::middleware::AuthUser;
use crate::models::tag::Tag;
use crate::repositories::TagRepo;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(list).post(create))
}

async fn list(State(state): State<AppState>) -> AppResult<Json<ApiResponse<Vec<Tag>>>> {
    Ok(Json(ApiResponse::new(TagRepo::list(&state.db).await?)))
}

async fn create(
    State(state): State<AppState>,
    AuthUser(_user): AuthUser,
    Json(body): Json<CreateTagRequest>,
) -> AppResult<Json<ApiResponse<Tag>>> {
    body.validate()?;
    let tag = TagRepo::find_or_create(&state.db, &body.name, &slugify(&body.name)).await?;
    Ok(Json(ApiResponse::new(tag)))
}
