use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;

pub async fn log(
    pool: &PgPool,
    user_id: Option<Uuid>,
    action: &str,
    resource: &str,
    resource_id: Option<&str>,
    details: Option<&str>,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO audit_logs (user_id, action, resource, resource_id, details)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(user_id)
    .bind(action)
    .bind(resource)
    .bind(resource_id)
    .bind(details)
    .execute(pool)
    .await?;
    Ok(())
}
