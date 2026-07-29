use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::generate_token;
use crate::config::Settings;
use crate::error::{AppError, AppResult};
use crate::models::{LoginRequest, LoginResponse, User, UserPublic};
use crate::services::audit;

pub async fn login(pool: &PgPool, settings: &Settings, req: LoginRequest) -> AppResult<LoginResponse> {
    let login = req
        .email
        .as_deref()
        .or(req.username.as_deref())
        .ok_or_else(|| AppError::Validation("email or username required".into()))?
        .trim()
        .to_string();

    if req.password.is_empty() {
        return Err(AppError::Validation("password required".into()));
    }

    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT * FROM users
        WHERE deleted_at IS NULL
          AND (LOWER(email) = LOWER($1) OR LOWER(username) = LOWER($1))
        LIMIT 1
        "#,
    )
    .bind(&login)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::Unauthorized("invalid credentials".into()))?;

    if !user.is_active {
        return Err(AppError::Forbidden("account disabled".into()));
    }

    let ok = bcrypt::verify(&req.password, &user.password_hash)
        .map_err(|e| AppError::Other(anyhow::anyhow!(e)))?;
    if !ok {
        return Err(AppError::Unauthorized("invalid credentials".into()));
    }

    sqlx::query("UPDATE users SET last_login = NOW(), updated_at = NOW() WHERE id = $1")
        .bind(user.id)
        .execute(pool)
        .await?;

    let token = generate_token(
        &user.id.to_string(),
        &user.email,
        &user.role,
        &settings.jwt_secret,
        &settings.jwt_issuer,
        settings.jwt_ttl_secs,
    )?;

    audit::log(
        pool,
        Some(user.id),
        "login",
        "user",
        Some(&user.id.to_string()),
        None,
    )
    .await?;

    Ok(LoginResponse {
        access_token: token,
        token_type: "Bearer".into(),
        user: UserPublic::from(user),
    })
}

pub async fn seed_admin(pool: &PgPool, settings: &Settings) -> AppResult<Option<Uuid>> {
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE deleted_at IS NULL")
        .fetch_one(pool)
        .await?;
    if count.0 > 0 {
        return Ok(None);
    }

    let hash = bcrypt::hash(&settings.seed_admin_password, bcrypt::DEFAULT_COST)
        .map_err(|e| AppError::Other(anyhow::anyhow!(e)))?;

    let id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO users (email, username, password_hash, role, name)
        VALUES ($1, $2, $3, 'admin', $4)
        RETURNING id
        "#,
    )
    .bind(&settings.seed_admin_email)
    .bind(&settings.seed_admin_username)
    .bind(hash)
    .bind(&settings.seed_admin_name)
    .fetch_one(pool)
    .await?;

    Ok(Some(id))
}

pub async fn get_user(pool: &PgPool, id: Uuid) -> AppResult<User> {
    sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::NotFound("user not found".into()))
}
