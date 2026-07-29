use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;

use crate::error::{AppError, AppResult};
use crate::models::user::{User, UserRole};
use crate::repositories::UserRepo;
use crate::services::auth::AuthService;
use crate::state::AppState;

/// Authenticated user extracted from Bearer JWT.
#[derive(Debug, Clone)]
pub struct AuthUser(pub User);

impl AuthUser {
    pub fn id(&self) -> uuid::Uuid {
        self.0.id
    }

    pub fn role(&self) -> UserRole {
        self.0.role
    }
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| AppError::Unauthorized("missing bearer token".into()))?;

        let claims = AuthService::decode_access_token(state, bearer.token())?;
        let user = UserRepo::find_by_id(&state.db, claims.sub)
            .await?
            .ok_or_else(|| AppError::Unauthorized("user not found".into()))?;
        if !user.is_active {
            return Err(AppError::Forbidden("account disabled".into()));
        }
        Ok(AuthUser(user))
    }
}

/// Optional auth — missing/invalid token yields None rather than 401.
#[derive(Debug, Clone)]
pub struct OptionalAuthUser(pub Option<User>);

impl FromRequestParts<AppState> for OptionalAuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        match TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state).await {
            Ok(TypedHeader(Authorization(bearer))) => {
                match AuthService::decode_access_token(state, bearer.token()) {
                    Ok(claims) => {
                        let user = UserRepo::find_by_id(&state.db, claims.sub).await?;
                        Ok(OptionalAuthUser(user.filter(|u| u.is_active)))
                    }
                    Err(_) => Ok(OptionalAuthUser(None)),
                }
            }
            Err(_) => Ok(OptionalAuthUser(None)),
        }
    }
}

pub struct RequireAdmin(pub User);

impl FromRequestParts<AppState> for RequireAdmin {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let AuthUser(user) = AuthUser::from_request_parts(parts, state).await?;
        if user.role != UserRole::Admin {
            return Err(AppError::Forbidden("admin required".into()));
        }
        Ok(RequireAdmin(user))
    }
}

pub struct RequireEditor(pub User);

impl FromRequestParts<AppState> for RequireEditor {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let AuthUser(user) = AuthUser::from_request_parts(parts, state).await?;
        if !user.role.can_moderate() {
            return Err(AppError::Forbidden("editor or admin required".into()));
        }
        Ok(RequireEditor(user))
    }
}

#[allow(dead_code)]
pub fn ensure_role(user: &User, min: UserRole) -> AppResult<()> {
    let ok = match min {
        UserRole::Admin => user.role == UserRole::Admin,
        UserRole::Editor => user.role.can_moderate(),
        UserRole::Author => user.role.can_edit(),
        UserRole::Viewer => true,
    };
    if ok {
        Ok(())
    } else {
        Err(AppError::Forbidden("insufficient permissions".into()))
    }
}
