use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use validator::Validate;

use crate::dto::cases::{
    AssignCaseRequest, CaseDetail, CaseListQuery, CaseStats, CreateCaseNoteRequest,
    CreateCaseRequest, InvestigatorCaseStats, UpdateCaseRequest,
};
use crate::dto::common::{ApiResponse, MessageResponse, Paginated};
use crate::error::AppResult;
use crate::middleware::AuthUser;
use crate::models::case::{Case, CaseActivity, CaseNote};
use crate::services::cases::CaseService;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_cases).post(create_case))
        .route("/stats", get(get_stats))
        .route("/investigator-stats", get(get_investigator_stats))
        .route(
            "/{id_or_number}",
            get(get_case).put(update_case).delete(delete_case),
        )
        .route("/{id_or_number}/assign", axum::routing::post(assign_case))
        .route(
            "/{id_or_number}/notes",
            get(list_notes).post(add_note),
        )
        .route("/{id_or_number}/activities", get(list_activities))
}

#[utoipa::path(
    get,
    path = "/api/v1/cases",
    tag = "cases",
    security(("bearer" = [])),
    params(CaseListQuery),
    responses((status = 200, description = "List cases"))
)]
pub async fn list_cases(
    State(state): State<AppState>,
    AuthUser(_user): AuthUser,
    Query(q): Query<CaseListQuery>,
) -> AppResult<Json<ApiResponse<Paginated<Case>>>> {
    let data = CaseService::list(&state, q).await?;
    Ok(Json(ApiResponse::new(data)))
}

#[utoipa::path(
    post,
    path = "/api/v1/cases",
    tag = "cases",
    security(("bearer" = [])),
    request_body = CreateCaseRequest,
    responses((status = 200, description = "Create case"))
)]
pub async fn create_case(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<CreateCaseRequest>,
) -> AppResult<Json<ApiResponse<Case>>> {
    body.validate()?;
    let case = CaseService::create(&state, &user, body).await?;
    Ok(Json(ApiResponse::new(case)))
}

#[utoipa::path(
    get,
    path = "/api/v1/cases/{id_or_number}",
    tag = "cases",
    security(("bearer" = [])),
    params(("id_or_number" = String, Path, description = "UUID or Ace case number")),
    responses((status = 200, description = "Case detail"))
)]
pub async fn get_case(
    State(state): State<AppState>,
    AuthUser(_user): AuthUser,
    Path(id_or_number): Path<String>,
) -> AppResult<Json<ApiResponse<CaseDetail>>> {
    let detail = CaseService::get(&state, &id_or_number).await?;
    Ok(Json(ApiResponse::new(detail)))
}

#[utoipa::path(
    put,
    path = "/api/v1/cases/{id_or_number}",
    tag = "cases",
    security(("bearer" = [])),
    params(("id_or_number" = String, Path, description = "UUID or Ace case number")),
    request_body = UpdateCaseRequest,
    responses((status = 200, description = "Updated case"))
)]
pub async fn update_case(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id_or_number): Path<String>,
    Json(body): Json<UpdateCaseRequest>,
) -> AppResult<Json<ApiResponse<Case>>> {
    body.validate()?;
    let case = CaseService::update(&state, &user, &id_or_number, body).await?;
    Ok(Json(ApiResponse::new(case)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/cases/{id_or_number}",
    tag = "cases",
    security(("bearer" = [])),
    params(("id_or_number" = String, Path, description = "UUID or Ace case number")),
    responses((status = 200, description = "Deleted"))
)]
pub async fn delete_case(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id_or_number): Path<String>,
) -> AppResult<Json<MessageResponse>> {
    CaseService::delete(&state, &user, &id_or_number).await?;
    Ok(Json(MessageResponse {
        message: "case deleted".into(),
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/cases/{id_or_number}/assign",
    tag = "cases",
    security(("bearer" = [])),
    params(("id_or_number" = String, Path, description = "UUID or Ace case number")),
    request_body = AssignCaseRequest,
    responses((status = 200, description = "Assigned case"))
)]
pub async fn assign_case(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id_or_number): Path<String>,
    Json(body): Json<AssignCaseRequest>,
) -> AppResult<Json<ApiResponse<Case>>> {
    body.validate()?;
    let case = CaseService::assign(&state, &user, &id_or_number, body).await?;
    Ok(Json(ApiResponse::new(case)))
}

#[utoipa::path(
    get,
    path = "/api/v1/cases/{id_or_number}/notes",
    tag = "cases",
    security(("bearer" = [])),
    params(("id_or_number" = String, Path, description = "UUID or Ace case number")),
    responses((status = 200, description = "Notes"))
)]
pub async fn list_notes(
    State(state): State<AppState>,
    AuthUser(_user): AuthUser,
    Path(id_or_number): Path<String>,
) -> AppResult<Json<ApiResponse<Vec<CaseNote>>>> {
    let notes = CaseService::list_notes(&state, &id_or_number).await?;
    Ok(Json(ApiResponse::new(notes)))
}

#[utoipa::path(
    post,
    path = "/api/v1/cases/{id_or_number}/notes",
    tag = "cases",
    security(("bearer" = [])),
    params(("id_or_number" = String, Path, description = "UUID or Ace case number")),
    request_body = CreateCaseNoteRequest,
    responses((status = 200, description = "Created note"))
)]
pub async fn add_note(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id_or_number): Path<String>,
    Json(body): Json<CreateCaseNoteRequest>,
) -> AppResult<Json<ApiResponse<CaseNote>>> {
    body.validate()?;
    let note = CaseService::add_note(&state, &user, &id_or_number, body).await?;
    Ok(Json(ApiResponse::new(note)))
}

#[utoipa::path(
    get,
    path = "/api/v1/cases/{id_or_number}/activities",
    tag = "cases",
    security(("bearer" = [])),
    params(("id_or_number" = String, Path, description = "UUID or Ace case number")),
    responses((status = 200, description = "Activities"))
)]
pub async fn list_activities(
    State(state): State<AppState>,
    AuthUser(_user): AuthUser,
    Path(id_or_number): Path<String>,
) -> AppResult<Json<ApiResponse<Vec<CaseActivity>>>> {
    let activities = CaseService::list_activities(&state, &id_or_number).await?;
    Ok(Json(ApiResponse::new(activities)))
}

#[utoipa::path(
    get,
    path = "/api/v1/cases/stats",
    tag = "cases",
    security(("bearer" = [])),
    responses((status = 200, description = "Case status counts"))
)]
pub async fn get_stats(
    State(state): State<AppState>,
    AuthUser(_user): AuthUser,
) -> AppResult<Json<ApiResponse<CaseStats>>> {
    let stats = CaseService::stats(&state).await?;
    Ok(Json(ApiResponse::new(stats)))
}

#[utoipa::path(
    get,
    path = "/api/v1/cases/investigator-stats",
    tag = "cases",
    security(("bearer" = [])),
    responses((status = 200, description = "Per-investigator counts"))
)]
pub async fn get_investigator_stats(
    State(state): State<AppState>,
    AuthUser(_user): AuthUser,
) -> AppResult<Json<ApiResponse<Vec<InvestigatorCaseStats>>>> {
    let stats = CaseService::investigator_stats(&state).await?;
    Ok(Json(ApiResponse::new(stats)))
}
