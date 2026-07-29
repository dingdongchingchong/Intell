use axum::extract::{Query, State};
use axum::response::Redirect;
use axum::routing::{get, post};
use axum::{Json, Router};
use validator::Validate;

use crate::dto::auth::*;
use crate::dto::common::{ApiResponse, MessageResponse};
use crate::error::{AppError, AppResult};
use crate::middleware::AuthUser;
use crate::services::AuthService;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
        .route("/change-password", post(change_password))
        .route("/me", get(me))
        .route("/oauth/twitter", get(twitter_start))
        .route("/oauth/twitter/callback", get(twitter_callback))
}

async fn register(
    State(state): State<AppState>,
    Json(body): Json<RegisterRequest>,
) -> AppResult<Json<ApiResponse<AuthResponse>>> {
    body.validate()?;
    let auth = AuthService::register(
        &state,
        &body.email,
        &body.username,
        &body.password,
        &body.display_name,
    )
    .await?;
    Ok(Json(ApiResponse::new(auth)))
}

async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> AppResult<Json<ApiResponse<AuthResponse>>> {
    body.validate()?;
    let auth = AuthService::login(&state, &body.login, &body.password).await?;
    Ok(Json(ApiResponse::new(auth)))
}

async fn refresh(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> AppResult<Json<ApiResponse<TokenPair>>> {
    let tokens = AuthService::refresh(&state, &body.refresh_token).await?;
    Ok(Json(ApiResponse::new(tokens)))
}

async fn logout(
    State(state): State<AppState>,
    Json(body): Json<RefreshRequest>,
) -> AppResult<Json<MessageResponse>> {
    AuthService::logout(&state, &body.refresh_token).await?;
    Ok(Json(MessageResponse {
        message: "logged out".into(),
    }))
}

async fn change_password(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<ChangePasswordRequest>,
) -> AppResult<Json<MessageResponse>> {
    body.validate()?;
    AuthService::change_password(&state, user.id, &body.current_password, &body.new_password)
        .await?;
    Ok(Json(MessageResponse {
        message: "password updated".into(),
    }))
}

async fn me(AuthUser(user): AuthUser) -> Json<ApiResponse<crate::models::user::PublicUser>> {
    Json(ApiResponse::new(user.into()))
}

async fn twitter_start(State(state): State<AppState>) -> AppResult<Redirect> {
    if !state.settings.twitter_enabled() {
        return Err(AppError::BadRequest("Twitter OAuth not configured".into()));
    }
    let (url, _) = AuthService::twitter_auth_url(&state).await?;
    Ok(Redirect::temporary(&url))
}

async fn twitter_callback(
    State(state): State<AppState>,
    Query(q): Query<OAuthCallbackQuery>,
) -> AppResult<Redirect> {
    if let Some(err) = q.error {
        return Err(AppError::Unauthorized(err));
    }
    let code = q
        .code
        .ok_or_else(|| AppError::BadRequest("missing code".into()))?;
    let oauth_state = q
        .state
        .ok_or_else(|| AppError::BadRequest("missing state".into()))?;
    let auth = AuthService::twitter_callback(&state, &code, &oauth_state).await?;
    let redirect = format!(
        "{}/oauth/callback#access_token={}&refresh_token={}",
        state.settings.frontend_url, auth.tokens.access_token, auth.tokens.refresh_token
    );
    Ok(Redirect::temporary(&redirect))
}
