use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::notification::{Notification, NotificationType};

pub struct NotificationRepo;

impl NotificationRepo {
    pub async fn create(
        db: &PgPool,
        user_id: Uuid,
        actor_id: Option<Uuid>,
        r#type: NotificationType,
        title: &str,
        body: &str,
        entity_type: Option<&str>,
        entity_id: Option<Uuid>,
    ) -> AppResult<Notification> {
        Ok(sqlx::query_as::<_, Notification>(
            r#"INSERT INTO notifications (user_id, actor_id, type, title, body, entity_type, entity_id)
               VALUES ($1,$2,$3,$4,$5,$6,$7) RETURNING *"#,
        )
        .bind(user_id)
        .bind(actor_id)
        .bind(r#type)
        .bind(title)
        .bind(body)
        .bind(entity_type)
        .bind(entity_id)
        .fetch_one(db)
        .await?)
    }

    pub async fn list_for_user(
        db: &PgPool,
        user_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> AppResult<(Vec<Notification>, i64)> {
        let unread: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM notifications WHERE user_id = $1 AND is_read = FALSE",
        )
        .bind(user_id)
        .fetch_one(db)
        .await?;
        let items = sqlx::query_as::<_, Notification>(
            r#"SELECT * FROM notifications WHERE user_id = $1
               ORDER BY created_at DESC LIMIT $2 OFFSET $3"#,
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(db)
        .await?;
        Ok((items, unread.0))
    }

    pub async fn mark_read(db: &PgPool, user_id: Uuid, id: Uuid) -> AppResult<bool> {
        let res = sqlx::query(
            "UPDATE notifications SET is_read = TRUE WHERE id = $1 AND user_id = $2",
        )
        .bind(id)
        .bind(user_id)
        .execute(db)
        .await?;
        Ok(res.rows_affected() > 0)
    }

    pub async fn mark_all_read(db: &PgPool, user_id: Uuid) -> AppResult<u64> {
        let res = sqlx::query(
            "UPDATE notifications SET is_read = TRUE WHERE user_id = $1 AND is_read = FALSE",
        )
        .bind(user_id)
        .execute(db)
        .await?;
        Ok(res.rows_affected())
    }
}
