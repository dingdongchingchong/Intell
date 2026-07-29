use axum::extract::{Path, State};
use axum::routing::{delete, get};
use axum::{Json, Router};
use slug::slugify;
use uuid::Uuid;
use validator::Validate;

use crate::dto::common::{ApiResponse, MessageResponse};
use crate::dto::posts::CreateCategoryRequest;
use crate::error::AppResult;
use crate::middleware::RequireEditor;
use crate::models::category::Category;
use crate::repositories::CategoryRepo;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/{id}", delete(delete_one))
}

async fn list(State(state): State<AppState>) -> AppResult<Json<ApiResponse<Vec<Category>>>> {
    Ok(Json(ApiResponse::new(
        CategoryRepo::list(&state.db).await?,
    )))
}

async fn create(
    State(state): State<AppState>,
    RequireEditor(_): RequireEditor,
    Json(body): Json<CreateCategoryRequest>,
) -> AppResult<Json<ApiResponse<Category>>> {
    body.validate()?;
    let cat = CategoryRepo::create(
        &state.db,
        &body.name,
        &slugify(&body.name),
        body.description.as_deref().unwrap_or(""),
    )
    .await?;
    Ok(Json(ApiResponse::new(cat)))
}

async fn delete_one(
    State(state): State<AppState>,
    RequireEditor(_): RequireEditor,
    Path(id): Path<Uuid>,
) -> AppResult<Json<MessageResponse>> {
    CategoryRepo::delete(&state.db, id).await?;
    Ok(Json(MessageResponse {
        message: "category deleted".into(),
    }))
}
