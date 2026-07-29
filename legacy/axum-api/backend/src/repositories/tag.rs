use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;
use crate::models::tag::Tag;

pub struct TagRepo;

impl TagRepo {
    pub async fn find_or_create(db: &PgPool, name: &str, slug: &str) -> AppResult<Tag> {
        if let Some(existing) = sqlx::query_as::<_, Tag>("SELECT * FROM tags WHERE slug = $1")
            .bind(slug)
            .fetch_optional(db)
            .await?
        {
            return Ok(existing);
        }
        Ok(sqlx::query_as::<_, Tag>(
            "INSERT INTO tags (name, slug) VALUES ($1,$2)
             ON CONFLICT (slug) DO UPDATE SET name = EXCLUDED.name
             RETURNING *",
        )
        .bind(name)
        .bind(slug)
        .fetch_one(db)
        .await?)
    }

    pub async fn list(db: &PgPool) -> AppResult<Vec<Tag>> {
        Ok(sqlx::query_as::<_, Tag>("SELECT * FROM tags ORDER BY name")
            .fetch_all(db)
            .await?)
    }

    pub async fn for_post(db: &PgPool, post_id: Uuid) -> AppResult<Vec<Tag>> {
        Ok(sqlx::query_as::<_, Tag>(
            r#"SELECT t.* FROM tags t
               JOIN post_tags pt ON pt.tag_id = t.id
               WHERE pt.post_id = $1 ORDER BY t.name"#,
        )
        .bind(post_id)
        .fetch_all(db)
        .await?)
    }
}
