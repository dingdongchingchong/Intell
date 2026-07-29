use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::models::{CreateUserRequest, User, UserPublic, UserRole};
use crate::services::audit;

pub async fn list_users(pool: &PgPool) -> AppResult<Vec<UserPublic>> {
    let rows = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE deleted_at IS NULL ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(UserPublic::from).collect())
}

pub async fn create_user(pool: &PgPool, actor_id: Uuid, req: CreateUserRequest) -> AppResult<UserPublic> {
    let role = UserRole::parse(&req.role)
        .ok_or_else(|| AppError::Validation("invalid role".into()))?;

    if req.password.len() < 8 {
        return Err(AppError::Validation("password must be at least 8 characters".into()));
    }
    if req.email.trim().is_empty() || req.username.trim().is_empty() {
        return Err(AppError::Validation("email and username required".into()));
    }

    let hash = bcrypt::hash(&req.password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Other(anyhow::anyhow!(e)))?;

    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (email, username, password_hash, role, name)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING *
        "#,
    )
    .bind(req.email.trim().to_lowercase())
    .bind(req.username.trim())
    .bind(hash)
    .bind(role.as_str())
    .bind(req.name.trim())
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(db) if db.constraint().is_some() => {
            AppError::Conflict("email or username already exists".into())
        }
        other => AppError::from(other),
    })?;

    audit::log(
        pool,
        Some(actor_id),
        "user_create",
        "user",
        Some(&user.id.to_string()),
        Some(&user.email),
    )
    .await?;

    Ok(UserPublic::from(user))
}
