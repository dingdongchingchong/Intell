use axum::extract::{Path, Query, State};
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use serde::Deserialize;
use utoipa::IntoParams;
use uuid::Uuid;

use crate::dto::common::{ApiResponse, Paginated, PaginationQuery};
use crate::dto::users::{AdminUpdateUserRequest, UserListItem};
use crate::error::AppResult;
use crate::middleware::{RequireAdmin, RequireEditor};
use crate::models::post::{Post, PostStatus};
use crate::models::user::PublicUser;
use crate::services::admin::{AdminDashboard, AdminService};
use crate::services::{SshKeyEntry, SshKeyService};
use crate::state::AppState;

#[derive(Debug, Deserialize, IntoParams)]
pub struct AdminPostQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_per")]
    pub per_page: u32,
    pub status: Option<PostStatus>,
}

fn default_page() -> u32 {
    1
}
fn default_per() -> u32 {
    20
}

#[derive(Debug, Deserialize)]
pub struct ModerateBody {
    pub status: PostStatus,
}

#[derive(Debug, Deserialize)]
pub struct AddSshKeyRequest {
    pub username: String,
    #[serde(alias = "publicKey")]
    pub public_key: String,
}

#[derive(Debug, serde::Serialize)]
pub struct SshKeysResponse {
    pub path: String,
    pub users: Vec<SshKeyEntry>,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/dashboard", get(dashboard))
        .route("/users", get(list_users))
        .route("/users/{id}", patch(update_user))
        .route("/posts", get(list_posts))
        .route("/posts/{id}/moderate", patch(moderate_post))
        .route("/ssh-keys", get(list_ssh_keys).post(add_ssh_key))
        .route("/ssh-keys/{username}", delete(remove_ssh_key))
}

async fn dashboard(
    State(state): State<AppState>,
    RequireAdmin(_): RequireAdmin,
) -> AppResult<Json<ApiResponse<AdminDashboard>>> {
    Ok(Json(ApiResponse::new(
        AdminService::dashboard(&state).await?,
    )))
}

async fn list_users(
    State(state): State<AppState>,
    RequireAdmin(_): RequireAdmin,
    Query(q): Query<PaginationQuery>,
) -> AppResult<Json<ApiResponse<Paginated<UserListItem>>>> {
    Ok(Json(ApiResponse::new(
        AdminService::list_users(&state, q).await?,
    )))
}

async fn update_user(
    State(state): State<AppState>,
    RequireAdmin(actor): RequireAdmin,
    Path(id): Path<Uuid>,
    Json(body): Json<AdminUpdateUserRequest>,
) -> AppResult<Json<ApiResponse<PublicUser>>> {
    Ok(Json(ApiResponse::new(
        AdminService::update_user(&state, &actor, id, body).await?,
    )))
}

async fn list_posts(
    State(state): State<AppState>,
    RequireEditor(_): RequireEditor,
    Query(q): Query<AdminPostQuery>,
) -> AppResult<Json<ApiResponse<Paginated<Post>>>> {
    Ok(Json(ApiResponse::new(
        AdminService::list_posts(
            &state,
            PaginationQuery {
                page: q.page,
                per_page: q.per_page,
            },
            q.status,
        )
        .await?,
    )))
}

async fn moderate_post(
    State(state): State<AppState>,
    RequireEditor(actor): RequireEditor,
    Path(id): Path<Uuid>,
    Json(body): Json<ModerateBody>,
) -> AppResult<Json<ApiResponse<Post>>> {
    Ok(Json(ApiResponse::new(
        AdminService::moderate_post(&state, &actor, id, body.status).await?,
    )))
}

async fn list_ssh_keys(
    RequireAdmin(_): RequireAdmin,
) -> AppResult<Json<ApiResponse<SshKeysResponse>>> {
    let users = SshKeyService::list()?;
    Ok(Json(ApiResponse::new(SshKeysResponse {
        path: SshKeyService::keys_path().display().to_string(),
        users,
    })))
}

async fn add_ssh_key(
    RequireAdmin(_): RequireAdmin,
    Json(body): Json<AddSshKeyRequest>,
) -> AppResult<Json<ApiResponse<SshKeyEntry>>> {
    let entry = SshKeyService::add(&body.username, &body.public_key)?;
    Ok(Json(ApiResponse::new(entry)))
}

async fn remove_ssh_key(
    RequireAdmin(_): RequireAdmin,
    Path(username): Path<String>,
) -> AppResult<Json<ApiResponse<serde_json::Value>>> {
    SshKeyService::remove(&username)?;
    Ok(Json(ApiResponse::new(serde_json::json!({
        "removed": username
    }))))
}
