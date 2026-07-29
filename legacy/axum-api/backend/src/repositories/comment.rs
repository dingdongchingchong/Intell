use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::comment::Comment;

pub struct CommentRepo;

impl CommentRepo {
    pub async fn create(
        db: &PgPool,
        post_id: Uuid,
        author_id: Uuid,
        parent_id: Option<Uuid>,
        body: &str,
    ) -> AppResult<Comment> {
        let mut tx = db.begin().await?;
        let comment = sqlx::query_as::<_, Comment>(
            r#"INSERT INTO comments (post_id, author_id, parent_id, body)
               VALUES ($1,$2,$3,$4) RETURNING *"#,
        )
        .bind(post_id)
        .bind(author_id)
        .bind(parent_id)
        .bind(body)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query("UPDATE posts SET comment_count = comment_count + 1 WHERE id = $1")
            .bind(post_id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(comment)
    }

    pub async fn list_for_post(
        db: &PgPool,
        post_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> AppResult<(Vec<Comment>, i64)> {
        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM comments WHERE post_id = $1 AND is_deleted = FALSE",
        )
        .bind(post_id)
        .fetch_one(db)
        .await?;
        let items = sqlx::query_as::<_, Comment>(
            r#"SELECT * FROM comments
               WHERE post_id = $1 AND is_deleted = FALSE
               ORDER BY created_at ASC LIMIT $2 OFFSET $3"#,
        )
        .bind(post_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(db)
        .await?;
        Ok((items, total.0))
    }

    pub async fn soft_delete(db: &PgPool, id: Uuid) -> AppResult<Option<Comment>> {
        let comment = sqlx::query_as::<_, Comment>(
            r#"UPDATE comments SET is_deleted = TRUE, body = '[deleted]', updated_at = NOW()
               WHERE id = $1 RETURNING *"#,
        )
        .bind(id)
        .fetch_optional(db)
        .await?;
        Ok(comment)
    }

    pub async fn find_by_id(db: &PgPool, id: Uuid) -> AppResult<Option<Comment>> {
        Ok(
            sqlx::query_as::<_, Comment>("SELECT * FROM comments WHERE id = $1")
                .bind(id)
                .fetch_optional(db)
                .await?,
        )
    }
}
