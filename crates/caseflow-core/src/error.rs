use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("config: {0}")]
    Config(String),
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error("forbidden: {0}")]
    Forbidden(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("validation: {0}")]
    Validation(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error(transparent)]
    Db(#[from] sqlx::Error),
    #[error(transparent)]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl AppError {
    pub fn status_code(&self) -> u16 {
        match self {
            Self::Unauthorized(_) => 401,
            Self::Forbidden(_) => 403,
            Self::NotFound(_) => 404,
            Self::Validation(_) => 400,
            Self::Conflict(_) => 409,
            Self::Config(_) => 500,
            Self::Db(_) | Self::Jwt(_) | Self::Other(_) => 500,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::Unauthorized(_) => "unauthorized",
            Self::Forbidden(_) => "forbidden",
            Self::NotFound(_) => "not_found",
            Self::Validation(_) => "validation_error",
            Self::Conflict(_) => "conflict",
            Self::Config(_) => "config_error",
            Self::Db(_) => "database_error",
            Self::Jwt(_) => "jwt_error",
            Self::Other(_) => "internal_error",
        }
    }
}
