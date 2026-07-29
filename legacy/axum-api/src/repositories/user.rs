use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::user::{User, UserRole};

pub struct UserRepo;

impl UserRepo {
    pub async fn find_by_id(db: &PgPool, id: Uuid) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"SELECT id, email, username, password_hash, display_name, bio, avatar_url,
                      role, is_active, is_verified, twitter_id, last_login_at, created_at, updated_at
               FROM users WHERE id = $1"#,
        )
        .bind(id)
        .fetch_optional(db)
        .await?;
        Ok(user)
    }

    pub async fn find_by_email(db: &PgPool, email: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"SELECT id, email, username, password_hash, display_name, bio, avatar_url,
                      role, is_active, is_verified, twitter_id, last_login_at, created_at, updated_at
               FROM users WHERE lower(email) = lower($1)"#,
        )
        .bind(email)
        .fetch_optional(db)
        .await?;
        Ok(user)
    }

    pub async fn find_by_username(db: &PgPool, username: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"SELECT id, email, username, password_hash, display_name, bio, avatar_url,
                      role, is_active, is_verified, twitter_id, last_login_at, created_at, updated_at
               FROM users WHERE lower(username) = lower($1)"#,
        )
        .bind(username)
        .fetch_optional(db)
        .await?;
        Ok(user)
    }

    pub async fn find_by_login(db: &PgPool, login: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"SELECT id, email, username, password_hash, display_name, bio, avatar_url,
                      role, is_active, is_verified, twitter_id, last_login_at, created_at, updated_at
               FROM users WHERE lower(email) = lower($1) OR lower(username) = lower($1)"#,
        )
        .bind(login)
        .fetch_optional(db)
        .await?;
        Ok(user)
    }

    pub async fn find_by_twitter_id(db: &PgPool, twitter_id: &str) -> AppResult<Option<User>> {
        let user = sqlx::query_as::<_, User>(
            r#"SELECT id, email, username, password_hash, display_name, bio, avatar_url,
                      role, is_active, is_verified, twitter_id, last_login_at, created_at, updated_at
               FROM users WHERE twitter_id = $1"#,
        )
        .bind(twitter_id)
        .fetch_optional(db)
        .await?;
        Ok(user)
    }

    pub async fn create(
        db: &PgPool,
        email: &str,
        username: &str,
        password_hash: Option<&str>,
        display_name: &str,
        role: UserRole,
        twitter_id: Option<&str>,
        is_verified: bool,
    ) -> AppResult<User> {
        let user = sqlx::query_as::<_, User>(
            r#"INSERT INTO users (email, username, password_hash, display_name, role, twitter_id, is_verified)
               VALUES ($1, $2, $3, $4, $5, $6, $7)
               RETURNING id, email, username, password_hash, display_name, bio, avatar_url,
                         role, is_active, is_verified, twitter_id, last_login_at, created_at, updated_at"#,
        )
        .bind(email)
        .bind(username)
        .bind(password_hash)
        .bind(display_name)
        .bind(role)
        .bind(twitter_id)
        .bind(is_verified)
        .fetch_one(db)
        .await?;
        Ok(user)
    }

    pub async fn update_profile(
        db: &PgPool,
        id: Uuid,
        display_name: Option<&str>,
        bio: Option<&str>,
        avatar_url: Option<&str>,
    ) -> AppResult<User> {
        let user = sqlx::query_as::<_, User>(
            r#"UPDATE users SET
                 display_name = COALESCE($2, display_name),
                 bio = COALESCE($3, bio),
                 avatar_url = COALESCE($4, avatar_url),
                 updated_at = NOW()
               WHERE id = $1
               RETURNING id, email, username, password_hash, display_name, bio, avatar_url,
                         role, is_active, is_verified, twitter_id, last_login_at, created_at, updated_at"#,
        )
        .bind(id)
        .bind(display_name)
        .bind(bio)
        .bind(avatar_url)
        .fetch_one(db)
        .await?;
        Ok(user)
    }

    pub async fn admin_update(
        db: &PgPool,
        id: Uuid,
        role: Option<UserRole>,
        is_active: Option<bool>,
        display_name: Option<&str>,
        password_hash: Option<&str>,
    ) -> AppResult<User> {
        let user = sqlx::query_as::<_, User>(
            r#"UPDATE users SET
                 role = COALESCE($2, role),
                 is_active = COALESCE($3, is_active),
                 display_name = COALESCE($4, display_name),
                 password_hash = COALESCE($5, password_hash),
                 updated_at = NOW()
               WHERE id = $1
               RETURNING id, email, username, password_hash, display_name, bio, avatar_url,
                         role, is_active, is_verified, twitter_id, last_login_at, created_at, updated_at"#,
        )
        .bind(id)
        .bind(role)
        .bind(is_active)
        .bind(display_name)
        .bind(password_hash)
        .fetch_one(db)
        .await?;
        Ok(user)
    }

    pub async fn set_password(db: &PgPool, id: Uuid, password_hash: &str) -> AppResult<()> {
        sqlx::query("UPDATE users SET password_hash = $2, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .bind(password_hash)
            .execute(db)
            .await?;
        Ok(())
    }

    pub async fn touch_login(db: &PgPool, id: Uuid) -> AppResult<()> {
        sqlx::query("UPDATE users SET last_login_at = NOW(), updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(db)
            .await?;
        Ok(())
    }

    pub async fn list(
        db: &PgPool,
        limit: i64,
        offset: i64,
    ) -> AppResult<(Vec<User>, i64)> {
        let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
            .fetch_one(db)
            .await?;
        let users = sqlx::query_as::<_, User>(
            r#"SELECT id, email, username, password_hash, display_name, bio, avatar_url,
                      role, is_active, is_verified, twitter_id, last_login_at, created_at, updated_at
               FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2"#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(db)
        .await?;
        Ok((users, total.0))
    }

    pub async fn count_admins(db: &PgPool) -> AppResult<i64> {
        let (n,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM users WHERE role = 'admin' AND is_active = TRUE")
                .fetch_one(db)
                .await?;
        Ok(n)
    }

    pub async fn store_refresh_token(
        db: &PgPool,
        user_id: Uuid,
        token_hash: &str,
        expires_at: DateTime<Utc>,
    ) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO refresh_tokens (user_id, token_hash, expires_at) VALUES ($1, $2, $3)",
        )
        .bind(user_id)
        .bind(token_hash)
        .bind(expires_at)
        .execute(db)
        .await?;
        Ok(())
    }

    pub async fn revoke_refresh_token(db: &PgPool, token_hash: &str) -> AppResult<bool> {
        let res = sqlx::query(
            "UPDATE refresh_tokens SET revoked_at = NOW() WHERE token_hash = $1 AND revoked_at IS NULL",
        )
        .bind(token_hash)
        .execute(db)
        .await?;
        Ok(res.rows_affected() > 0)
    }

    pub async fn find_valid_refresh(
        db: &PgPool,
        token_hash: &str,
    ) -> AppResult<Option<(Uuid, DateTime<Utc>)>> {
        let row: Option<(Uuid, DateTime<Utc>)> = sqlx::query_as(
            r#"SELECT user_id, expires_at FROM refresh_tokens
               WHERE token_hash = $1 AND revoked_at IS NULL AND expires_at > NOW()"#,
        )
        .bind(token_hash)
        .fetch_optional(db)
        .await?;
        Ok(row)
    }

    pub async fn save_oauth_state(
        db: &PgPool,
        state: &str,
        code_verifier: Option<&str>,
        redirect_uri: Option<&str>,
        expires_at: DateTime<Utc>,
    ) -> AppResult<()> {
        sqlx::query(
            "INSERT INTO oauth_states (state, code_verifier, redirect_uri, expires_at) VALUES ($1,$2,$3,$4)",
        )
        .bind(state)
        .bind(code_verifier)
        .bind(redirect_uri)
        .bind(expires_at)
        .execute(db)
        .await?;
        Ok(())
    }

    pub async fn take_oauth_state(
        db: &PgPool,
        state: &str,
    ) -> AppResult<Option<(Option<String>, Option<String>)>> {
        let row: Option<(Option<String>, Option<String>)> = sqlx::query_as(
            r#"DELETE FROM oauth_states WHERE state = $1 AND expires_at > NOW()
               RETURNING code_verifier, redirect_uri"#,
        )
        .bind(state)
        .fetch_optional(db)
        .await?;
        Ok(row)
    }
}
